use futures::future::join_all;
use tokio::time::sleep;
use std::time::Duration;
use std::future::Future;
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

async fn run_monitor_process<F, Fut>(network: Network, sender: Sender<GetFromInfuraParams>, process_log: F) -> Result<(), String>
    where
        F: Fn(Log) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    info!("Will be monitored: {}", network.name);

    let provider = Provider::<Http>::try_from(network.link)
        .map_err(|err| utils::make_err(Box::new(err), "create provider"))?;

    let mut last_block_number = get_last_block(&provider, &sender).await?;

    loop {
        let new_block_number = get_last_block(&provider, &sender).await?;
        if new_block_number <= last_block_number {
            continue;
        }

        let (resp_tx, resp_rx) = bounded(1);
        sender
            .send(GetFromInfuraParams::new(
                GetObject::GetLogs(last_block_number, new_block_number),
                provider.clone(),
                resp_tx
            )).await
            .map_err(|err| utils::make_err(Box::new(err), "send get_obj_result"))?;
        let logs = resp_rx.recv().await
            .map_err(|err| utils::make_err(Box::new(err), "receive response"))?
            .and_then(|obj| match obj {
                GottenObject::Logs(logs) => Ok(logs),
                _ => Err("Invalid object type: Expected a Vec<Log>".to_string()),
            })?;

        for log in logs {
            if let Err(err) = process_log(log).await {
                error!("Failed process event: {}", err);
            }
        }

        last_block_number = new_block_number;
    }
}

async fn get_last_block(provider: &Provider<Http>, sender: &Sender<GetFromInfuraParams>) -> Result<U64, String> {
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

    async fn run(&self, db: Arc<DB>, network: Network, sender: Sender<GetFromInfuraParams>) -> Result<(), String> {
        match self {
            Env::Test => run_monitor_process(network, sender, events::just_print_log).await,
            Env::Real => run_monitor_process(network, sender, move |log| {
                let db = db.clone();
                events::process_log(db, log)
            }).await,
        }
    }
}

pub async fn run_monitor(test: bool, networks: Vec<Network>, db: Arc<DB>) -> Result<(), String> {
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;

    let env = Env::new(test);

    let (req_tx, req_rx) = unbounded();

    tokio::spawn(request_handler(req_rx, Filter::new().event(&event_signature)));

    let tasks = networks
        .into_iter()
        .map(|n| {
            let env = env.clone();
            let db = db.clone();
            let req_tx = req_tx.clone();
            tokio::spawn(async move {
                env.run(db, n, req_tx).await
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

async fn request_handler(req_rx: Receiver<GetFromInfuraParams>, base_filter: Filter) -> Result<(), String> {
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

        sleep_duration(60).await;
    };
}