use std::sync::Arc;
use std::time::Duration;
use async_rate_limit::limiters::ThreadsafeVariableRateLimiter;
use async_rate_limit::sliding_window::SlidingWindowRateLimiter;
use ethers::middleware::Middleware;
use ethers::prelude::{Http, Provider, U64};
use ethers::types::{Filter, Log};
use tracing::{error, info};
use crate::{events, utils};
use crate::api::state::DB;
use crate::monitoring::app_state::MonitorAppState;
use crate::network::Network;
use crate::telegram::TelegramClient;

const CREDITS_PER_SECOND: usize = 500;
const CREDITS_PER_DAY: usize = 3_000_000;
const COST_GET_BLOCK_NUMBER: usize = 80;
const COST_GET_LOGS: usize = 255;

struct Limiter {
    rate_limiters: Vec<RateLimiter>,
}

enum RateLimiter {
    One(SlidingWindowRateLimiter),
    Cost(SlidingWindowRateLimiter),
}

impl Limiter {
    fn infura_limiter(rpm: f64) -> Self {
        let self_seconds = (60.0 / rpm).ceil() as u64;
        let one = RateLimiter::One(
            SlidingWindowRateLimiter::new(Duration::from_secs(self_seconds), 1)
        );

        let infura_second = RateLimiter::Cost(
            SlidingWindowRateLimiter::new(Duration::from_secs(24 * 3600), CREDITS_PER_DAY)
        );

        let infura_day = RateLimiter::Cost(
            SlidingWindowRateLimiter::new(Duration::from_secs(1), CREDITS_PER_SECOND)
        );

        Limiter::new(vec![one, infura_second, infura_day])
    }

    fn new(rate_limiters: Vec<RateLimiter>) -> Self {
        Self { rate_limiters }
    }

    async fn wait(&self, cost: usize) {
        for rate_limiter in &self.rate_limiters {
            match rate_limiter {
                RateLimiter::One(rl) => rl.wait_with_cost(1),
                RateLimiter::Cost(rl) => rl.wait_with_cost(cost)
            }.await
        }
    }
}

#[derive(Clone)]
enum Mode {
    Test,
    Real,
}

impl Mode {
    fn from_bool(value: bool) -> Self {
        Self::new(value)
    }

    fn new(test: bool) -> Self {
        if test { Mode::Test } else { Mode::Real }
    }

    async fn dispatch(&self, app_state: Arc<MonitorAppState>, log: &Log) -> Result<(), String> {
        match self {
            Self::Test => events::just_print_log(log).await,
            Self::Real => events::process_log(&app_state, log).await,
        }
    }
}

struct LogsRange {
    from: U64,
    to: U64,
}

impl LogsRange {
    fn new(from: U64, to: U64) -> Self {
        Self { from, to }
    }
}

enum InfuraRequest {
    Logs(LogsRange, Filter),
    LastBlockNumber,
}

impl InfuraRequest {
    fn cost(&self) -> usize {
        match self {
            Self::Logs(_, _) => COST_GET_LOGS,
            Self::LastBlockNumber => COST_GET_BLOCK_NUMBER,
        }
    }
}

enum InfuraResponse {
    Logs(Vec<Log>),
    BlockNumber(U64),
}

impl InfuraResponse {
    fn into_logs(self) -> Result<Vec<Log>, String> {
        match self {
            InfuraResponse::Logs(logs) => Ok(logs),
            InfuraResponse::BlockNumber(_) => Err("invalid type (block_number)".to_string())
        }
    }
    fn into_block_number(self) -> Result<U64, String> {
        match self {
            InfuraResponse::Logs(_) => Err("invalid type (block_number)".to_string()),
            InfuraResponse::BlockNumber(bn) => Ok(bn)
        }
    }
}

impl InfuraRequest {
    async fn into_response(self, provider: &Provider<Http>, limiter: &Limiter) -> Result<InfuraResponse, String> {
        limiter.wait(self.cost()).await;

        match self {
            Self::LastBlockNumber => provider
                .get_block_number()
                .await
                .map(InfuraResponse::BlockNumber)
                .map_err(|err| utils::make_err(Box::new(err), "get block number")),
            Self::Logs(range, base_filter) => provider
                .get_logs(&base_filter.from_block(range.from).to_block(range.to))
                .await
                .map(InfuraResponse::Logs)
                .map_err(|err| utils::make_err(Box::new(err), "get logs"))
        }
    }
}

enum LogsResult {
    Result(Vec<Log>, U64),
    Skip,
}

async fn get_logs_result(
    provider: &Provider<Http>,
    base_filter: &Filter,
    last_block_number: Option<U64>,
    limiter: &Limiter,
) -> Result<LogsResult, String> {
    let new_block_number = InfuraRequest::LastBlockNumber
        .into_response(provider, limiter)
        .await?
        .into_block_number()?;

    let last_block_number = match last_block_number {
        None => return Ok(LogsResult::Result(vec![], new_block_number)),
        Some(last_block_number) => last_block_number
    };

    if new_block_number <= last_block_number {
        return Ok(LogsResult::Skip);
    }

    InfuraRequest::Logs(
        LogsRange::new(last_block_number, new_block_number), base_filter.clone(),
    )
        .into_response(provider, limiter)
        .await?
        .into_logs()
        .map(|logs| LogsResult::Result(logs, new_block_number))
}

async fn get_network_logs(
    network: &Network, db: &DB, base_filter: &Filter, limiter: &Limiter,
) -> Result<Vec<Log>, String> {
    let last_block_number = db.get_block_number(&network.name)
        .await?
        .map(|bn| U64::from(bn));

    let provider = Provider::<Http>::try_from(&network.link)
        .map_err(|err| utils::make_err(Box::new(err), "create provider"))?;

    Ok(match get_logs_result(&provider, base_filter, last_block_number, &limiter)
        .await? {
        LogsResult::Skip => vec![],
        LogsResult::Result(logs, new_block_number) => {
            db.set_block_number(&network.name, new_block_number.as_u64() as i64).await?;
            logs
        }
    })
}

pub async fn process_networks(
    test: bool, networks: Vec<Network>, db: &DB, telegram_client: &TelegramClient
) -> Result<(), String> {
    let app_state = Arc::new(MonitorAppState::new(db.clone(), telegram_client.clone())?);

    let limiter = utils::get_env_or("INFRA_RPM", "1".to_string())?
        .parse::<u64>()
        .map(|rpm| Limiter::infura_limiter(rpm as f64))
        .map_err(|err| utils::make_err(Box::new(err), "parse infra rpm"))?;

    let base_filter = utils::get_env_var("EVENT_SIGNATURE")
        .map(|event_signature| Filter::new().event(&event_signature))?;

    loop {
        for network in &networks {
            info!("Will be monitored {}", network.name);
            let logs = get_network_logs(&network, &app_state.db, &base_filter, &limiter)
                .await
                .unwrap_or_else(|err| {
                    error!("Failed get_network_logs: {err}");
                    vec![]
                }
            );
            info!("Found {} logs in {}", logs.len(), network.name);

            for log in logs {
                let app_state_clone = app_state.clone();
                tokio::spawn(async move {
                    if let Err(err) = Mode::from_bool(test).dispatch(app_state_clone, &log)
                        .await {
                        error!("Failed dispatch log: {err}")
                    }
                });
            }
        }
    }
}
