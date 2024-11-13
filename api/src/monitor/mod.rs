use futures::future::join_all;
use tokio::time::sleep;
use std::time::Duration;
use std::sync::Arc;
use async_channel::{bounded, Receiver, Sender, unbounded};
use ethers::prelude::{Filter, Http, Provider, U64};
use ethers::providers::Middleware;
use ethers::types::Log;
use tracing::{error, info};
use uuid::Uuid;

use crate::{events, utils};
use crate::api::state::DB;
use crate::network::Network;
use crate::telegram::TelegramClient;

#[derive(Clone)]
pub struct MonitorAppState {
    pub db: DB,
    pub telegram_client: TelegramClient
}

impl MonitorAppState {
    fn new(db: DB, telegram_client: TelegramClient) -> Self {
        Self{ db, telegram_client }
    }
}

async fn get_last_block(db: &DB, network: &str, provider: &Provider<Http>, sender: &Sender<GetFromInfuraParams>) -> Result<U64, String> {
    Ok(match db.get_block_number(&network).await {
        Ok(Some(block_number)) => U64([block_number as u64]),
        need_get_new => {
            need_get_new?;
            let block = get_last_block_from_network(provider, sender).await?;
            db.set_block_number(&network, block.0[0] as i64).await?;
            block
        },
    })
}

async fn get_last_block_from_network(provider: &Provider<Http>, sender: &Sender<GetFromInfuraParams>) -> Result<U64, String> {
    let (resp_tx, resp_rx) = bounded(1);
    sender
        .send(GetFromInfuraParams::new(GetObject::GetLastBlock, provider.clone(), resp_tx))
        .await
        .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;

    resp_rx.recv().await
        .map_err(|err| utils::make_err(Box::new(err), "receive response"))?
        .and_then(|obj| match obj {
            GottenObject::Block(block) => Ok(block),
            _ => Err("Invalid object type: Expected a Block".to_string()),
        })
}

async fn sleep_duration(to_sleep: u64) {
    info!("Sleeping for {} seconds between logs check", to_sleep);
    sleep(Duration::from_secs(to_sleep)).await;
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

    async fn run(&self, app_state: Arc<MonitorAppState>, network: Network, sender: Sender<GetFromInfuraParams>) -> Result<(), String> {
        info!("Will be monitored: {}", network.name);

        let provider = Provider::<Http>::try_from(network.link)
            .map_err(|err| utils::make_err(Box::new(err), "create provider"))?;

        let mut last_block_number = get_last_block(&app_state.db, &network.name, &provider, &sender).await?;

        loop {
            let new_block_number = get_last_block_from_network(&provider, &sender).await?;
            if new_block_number <= last_block_number {
                continue;
            }

            let (resp_tx, resp_rx) = bounded(1);
            sender
                .send(GetFromInfuraParams::new(
                    GetObject::GetLogs(last_block_number, new_block_number),
                    provider.clone(),
                    resp_tx,
                )).await
                .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;
            let logs = resp_rx.recv().await
                .map_err(|err| utils::make_err(Box::new(err), "receive response"))?
                .and_then(|obj| match obj {
                    GottenObject::Logs(logs) => Ok(logs),
                    _ => Err("Invalid object type: Expected a Vec<Log>".to_string()),
                })?;

            for log in logs {
                if let Err(err) = self.process_log(app_state.clone(), log).await {
                    error!("Failed process event: {}", err);
                }
            }

            last_block_number = new_block_number;
            app_state.db.set_block_number(&network.name, last_block_number.0[0] as i64).await?;
        }
    }
}

pub async fn run_monitor(test: bool, networks: Vec<Network>, db: DB, telegram_client: TelegramClient) -> Result<(), String> {
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let rpm = utils::get_env_or("INFRA_RPM", "1".to_string())?;
    let rpm = rpm.parse::<i32>()
        .map_err(|err| utils::make_err(Box::new(err), "parse infra rpm"))?;

    let app_state = MonitorAppState::new(db, telegram_client);
    let env = Env::new(test);

    let (req_tx, req_rx) = unbounded();

    tokio::spawn(request_handler(req_rx, Filter::new().event(&event_signature), rpm));

    let tasks = networks
        .into_iter()
        .map(|n| {
            let env = env.clone();
            let req_tx = req_tx.clone();
            let app_state = app_state.clone();
            tokio::spawn(async move {
                env.run(Arc::new(app_state), n, req_tx).await
            })
        })
        .collect::<Vec<_>>();

    let results = join_all(tasks).await;

    for result in results {
        result.map_err(|e| format!("Task failed: {:?}", e))??;
    }

    Ok(())
}

struct GetFromInfuraParams {
    task_id: String,
    get_object: GetObject,
    provider: Provider<Http>,
    sender: Sender<Result<GottenObject, String>>,
}

impl GetFromInfuraParams {
    fn new(get_object: GetObject, provider: Provider<Http>, sender: Sender<Result<GottenObject, String>>) -> Self {
        let task_id = Uuid::new_v4().to_string();
        Self { task_id, get_object, provider, sender }
    }
}

enum GetObject {
    GetLogs(U64, U64),
    GetLastBlock,
}

#[derive(Debug)]
enum GottenObject {
    Logs(Vec<Log>),
    Block(U64),
}

async fn request_handler(req_rx: Receiver<GetFromInfuraParams>, base_filter: Filter, rpm: i32) -> Result<(), String> {
    loop {
        if let Ok(get_logs_params) = req_rx.recv().await {
            info!("Got task={}", get_logs_params.task_id);
            let get_obj_result = match get_logs_params.get_object {
                GetObject::GetLogs(block_from, block_to) => {
                    let filter = base_filter.clone()
                        .from_block(block_from)
                        .to_block(block_to);

                    get_logs_params.provider
                        .get_logs(&filter)
                        .await
                        .map_err(|err| utils::make_err(Box::new(err), "get logs"))
                        .map(GottenObject::Logs)
                }
                GetObject::GetLastBlock => {
                    get_logs_params.provider
                        .get_block_number()
                        .await
                        .map_err(|err| utils::make_err(Box::new(err), "get block number"))
                        .map(GottenObject::Block)
                }
            };

            get_logs_params.sender
                .send(get_obj_result)
                .await
                .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;
            info!("Finished task={}", get_logs_params.task_id);
        }

        sleep_duration((60 / rpm) as u64).await;
    };
}