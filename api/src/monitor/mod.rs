use std::collections::HashMap;
use futures::future::join_all;
use std::time::Duration;
use std::sync::Arc;
use async_channel::{bounded, Receiver, Sender, unbounded};
use async_rate_limit::limiters::VariableCostRateLimiter;
use async_rate_limit::sliding_window::SlidingWindowRateLimiter;
use ethers::prelude::{Filter, Http, Provider, U64};
use ethers::providers::Middleware;
use ethers::types::Log;
use tokio::time::sleep;
use tracing::{error, info};
use uuid::Uuid;

use crate::{events, utils};
use crate::api::state::DB;
use crate::network::Network;
use crate::mailer::Mailer;
use crate::telegram::TelegramClient;

const CREDITS_PER_SECOND: usize = 500;
const CREDITS_PER_DAY: usize = 3_000_000;
const COST_GET_BLOCK_NUMBER: usize = 80;
const COST_GET_LOGS: usize = 255;

#[derive(Clone)]
pub struct MonitorAppState {
    pub db: DB,
    pub telegram_client: TelegramClient,
    pub mailer: Mailer,
}

impl MonitorAppState {
    fn new(db: DB, telegram_client: TelegramClient) -> Result<Self, String> {
        let mailer = Mailer::new()?;
        Ok(Self { db, telegram_client, mailer })
    }
}

async fn get_last_block(
    db: &DB,
    network: &str,
    sender: &Sender<GetFromInfuraParams>,
    receiver: &Receiver<Result<GottenObject, String>>,
) -> Result<U64, String> {
    if let Some(block_number) = db.get_block_number(&network).await? {
        return Ok(U64([block_number as u64]));
    }

    let block = get_only_last_block_from_network(network, sender, receiver).await;
    db.set_block_number(&network, block.0[0] as i64).await?;
    Ok(block)
}


async fn get_only_last_block_from_network(
    network: &str,
    sender: &Sender<GetFromInfuraParams>,
    receiver: &Receiver<Result<GottenObject, String>>,
) -> U64 {
    loop {
        match get_last_block_from_network(network, sender, receiver).await {
            Ok(block) => return block,
            Err(err) => error!("Failed get_last_block_from_network: {}", err)
        };

        sleep(Duration::from_secs(1)).await;
    }
}

async fn get_last_block_from_network(
    network: &str,
    sender: &Sender<GetFromInfuraParams>,
    receiver: &Receiver<Result<GottenObject, String>>,
) -> Result<U64, String> {
    sender
        .send(GetFromInfuraParams::new(network, GetObject::GetLastBlock))
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;

    match receiver.recv().await {
        Ok(Ok(GottenObject::Block(block))) => Ok(block),
        Ok(Ok(GottenObject::Logs(_))) => Err("not block response".to_string()),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(utils::make_err(Box::new(err), "receive block response")),
    }
}

async fn get_only_logs(
    get_from_infura_params: &GetFromInfuraParams,
    sender: &Sender<GetFromInfuraParams>,
    receiver: &Receiver<Result<GottenObject, String>>,
) -> Vec<Log> {
    loop {
        match get_logs(get_from_infura_params, sender, receiver).await {
            Ok(logs) => return logs,
            Err(err) => error!("Failed get_logs: {}", err)
        };

        sleep(Duration::from_secs(1)).await;
    }
}

async fn get_logs(
    get_from_infura_params: &GetFromInfuraParams,
    sender: &Sender<GetFromInfuraParams>,
    receiver: &Receiver<Result<GottenObject, String>>,
) -> Result<Vec<Log>, String> {
    sender
        .send(get_from_infura_params.clone())
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;

    match receiver.recv().await {
        Ok(Ok(GottenObject::Block(_))) => Err("not logs response".to_string()),
        Ok(Ok(GottenObject::Logs(logs))) => Ok(logs),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(utils::make_err(Box::new(err), "receive logs response")),
    }
}

#[derive(Clone)]
enum Env {
    Test,
    Real,
}

impl Env {
    fn new(test: bool) -> Self {
        if test { Env::Test } else { Env::Real }
    }

    async fn process_log(&self, app_state: Arc<MonitorAppState>, log: Log) -> Result<(), String> {
        match self {
            Env::Test => events::just_print_log(log).await,
            Env::Real => events::process_log(app_state, log).await
        }
    }

    async fn run(
        &self,
        app_state: Arc<MonitorAppState>,
        network_name: String,
        sender: Sender<GetFromInfuraParams>,
        receiver: Receiver<Result<GottenObject, String>>,
    ) -> Result<(), String> {
        info!("Will be monitored: {}", network_name);

        let mut last_block_number = get_last_block(&app_state.db, &network_name, &sender, &receiver).await?;

        loop {
            let new_block_number = get_only_last_block_from_network(&network_name, &sender, &receiver).await;
            if new_block_number <= last_block_number {
                continue;
            }

            let logs = get_only_logs(
                &GetFromInfuraParams::new(
                    &network_name,
                    GetObject::GetLogs(last_block_number, new_block_number),
                ),
                &sender,
                &receiver,
            ).await;

            for log in logs {
                if let Err(err) = self.process_log(app_state.clone(), log).await {
                    error!("Failed process event: {}", err);
                }
            }

            last_block_number = new_block_number;
            app_state.db.set_block_number(&network_name, last_block_number.0[0] as i64).await?;
        }
    }
}

pub async fn run_monitor(test: bool, networks: Vec<Network>, db: DB, telegram_client: TelegramClient) -> Result<(), String> {
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let rpm = utils::get_env_or("INFRA_RPM", "1".to_string())?;
    let rpm = rpm.parse::<u64>()
        .map_err(|err| utils::make_err(Box::new(err), "parse infra rpm"))?;

    let app_state = Arc::new(MonitorAppState::new(db, telegram_client)?);
    let env = Env::new(test);

    let (req_tx, req_rx) = unbounded();

    let mut providers: HashMap<String, Provider<Http>> = HashMap::new();
    for n in networks.into_iter() {
        let provider = Provider::<Http>::try_from(&n.link)
            .map_err(|err| utils::make_err(Box::new(err), "create provider"))?;
        providers.insert(n.name.to_string(), provider);
    }

    let mut platform_hub: HashMap<String, NetworkPlatform> = HashMap::new();
    let mut tasks = providers
        .into_iter()
        .map(|(network, provider)| {
            let (resp_tx, resp_rx) = bounded(1);
            platform_hub.insert(network.clone(), NetworkPlatform::new(provider, resp_tx));
            let env = env.clone();
            let req_tx = req_tx.clone();
            let app_state = app_state.clone();
            tokio::spawn(async move {
                env.run(app_state, network, req_tx, resp_rx).await
            })
        })
        .collect::<Vec<_>>();

    tasks.push(
        tokio::spawn(request_handler(
            req_rx,
            platform_hub,
            Filter::new().event(&event_signature),
            rpm))
    );

    let results = join_all(tasks).await;

    for result in results {
        result.map_err(|e| format!("Task failed: {:?}", e))??;
    }

    Ok(())
}

struct NetworkPlatform {
    provider: Provider<Http>,
    sender: Sender<Result<GottenObject, String>>,
}

impl NetworkPlatform {
    fn new(provider: Provider<Http>, sender: Sender<Result<GottenObject, String>>) -> Self {
        Self { provider, sender }
    }
}

#[derive(Clone)]
struct GetFromInfuraParams {
    task_id: String,
    network: String,
    get_object: GetObject,
}

impl GetFromInfuraParams {
    fn new(network_name: &str, get_object: GetObject) -> Self {
        let task_id = Uuid::new_v4().to_string();
        let network = network_name.to_string();
        Self { task_id, network, get_object }
    }
}

#[derive(Clone)]
enum GetObject {
    GetLogs(U64, U64),
    GetLastBlock,
}

impl GetObject {
    fn cost(&self) -> usize {
        match self {
            GetObject::GetLogs(_, _) => COST_GET_LOGS,
            GetObject::GetLastBlock => COST_GET_BLOCK_NUMBER,
        }
    }
}

#[derive(Debug)]
enum GottenObject {
    Logs(Vec<Log>),
    Block(U64),
}

async fn request_handler(
    req_rx: Receiver<GetFromInfuraParams>,
    platform_hub: HashMap<String, NetworkPlatform>,
    base_filter: Filter,
    rpm: u64,
) -> Result<(), String> {
    let self_repeat_timeout = (60.0 / rpm as f64).ceil() as u64;
    info!("self_repeat_timeout = {self_repeat_timeout}");
    let mut self_limiter = SlidingWindowRateLimiter::new(
        Duration::from_secs(self_repeat_timeout), 1,
    );
    let mut infura_day_limiter = SlidingWindowRateLimiter::new(
        Duration::from_secs(24 * 3600), CREDITS_PER_DAY,
    );
    let mut infura_second_limiter = SlidingWindowRateLimiter::new(
        Duration::from_secs(1), CREDITS_PER_SECOND,
    );

    loop {
        if let Ok(get_logs_params) = req_rx.recv().await {
            info!("Got network={}, task={}", get_logs_params.network, get_logs_params.task_id);

            if let Some(platform) = platform_hub.get(&get_logs_params.network) {
                let cost = get_logs_params.get_object.cost();

                self_limiter.wait_with_cost(1).await;
                infura_second_limiter.wait_with_cost(cost).await;
                infura_day_limiter.wait_with_cost(cost).await;

                let get_obj_result = match get_logs_params.get_object {
                    GetObject::GetLogs(block_from, block_to) => {
                        let filter = base_filter.clone()
                            .from_block(block_from)
                            .to_block(block_to);

                        platform.provider
                            .get_logs(&filter)
                            .await
                            .map_err(|err| utils::make_err(Box::new(err), "get logs"))
                            .map(GottenObject::Logs)
                    }
                    GetObject::GetLastBlock => {
                        platform.provider
                            .get_block_number()
                            .await
                            .map_err(|err| utils::make_err(Box::new(err), "get block number"))
                            .map(GottenObject::Block)
                    }
                };

                platform.sender
                    .send(get_obj_result)
                    .await
                    .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;
            }

            info!("Finished network={}, task={}", get_logs_params.network, get_logs_params.task_id);
        }
    };
}