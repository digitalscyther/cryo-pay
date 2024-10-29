mod infura;

use futures::future::join_all;
use tokio::time::sleep;
use std::time::Duration;
use std::future::Future;
use std::sync::Arc;
use ethers::types::Log;
use tracing::{error, info};

use infura::{BlockGetter, LogsGetter, Monitor, MonitorFilter, MonitorProvider};
use crate::{events, utils};
use crate::api::state::DB;
use crate::network::Network;

pub async fn run_monitor_process<F, Fut>(network: Network, process_log: F) -> Result<(), String>
    where
        F: Fn(Log) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let delay_between_checks = utils::get_env_var("DELAY_BETWEEN_CHECKS")?
        .parse()
        .map_err(|err| {
            error!("foo: {}", err);
            utils::make_err(Box::new(err), "parse DELAY_BETWEEN_CHECKS")
        })?;
    info!("Will be monitored: {}", network.name);
    start_monitor(
        &network.link,
        &network.addresses.contract,
        &event_signature,
        delay_between_checks,
        process_log,
    ).await
}

async fn start_monitor<F, Fut>(
    provider_src: &str,
    address_str: &str,
    event_signature: &str,
    delay_between_checks: u64,
    process_log: F,
) -> Result<(), String>
    where
        F: Fn(Log) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    let monitor = Monitor::new(
        MonitorProvider::Url(provider_src), MonitorFilter::Signature(event_signature),
    )?.with_address(address_str)?;

    let mut last_block_number = monitor.get_block_number().await?;

    loop {
        sleep_duration(delay_between_checks).await;

        let new_block_number = monitor.get_block_number().await?;
        if new_block_number <= last_block_number {
            continue;
        }

        let logs = monitor.get_logs(last_block_number, new_block_number).await?;

        for log in logs {
            if let Err(err) = process_log(log).await {
                error!("Failed process event: {}", err);
            }
        }

        last_block_number = new_block_number;
    }
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

    async fn run(&self, db: Arc<DB>, network: Network) -> Result<(), String> {
        match self {
            Env::Test => run_monitor_process(network, events::just_print_log).await,
            Env::Real => run_monitor_process(network, move |log| {
                let db = db.clone();
                events::process_log(db, log)
            }).await,
        }
    }
}

pub async fn run_monitor(test: bool, networks: Vec<Network>, db: Arc<DB>) -> Result<(), String> {
    let env = Env::new(test);

    let tasks = networks
        .into_iter()
        .map(|n| {
            let env = env.clone();
            let db = db.clone();
            tokio::spawn(async move {
                env.run(db, n).await
            })
        })
        .collect::<Vec<_>>();

    let results = join_all(tasks).await;

    for result in results {
        result.map_err(|e| format!("Task failed: {:?}", e))??;
    }

    Ok(())
}