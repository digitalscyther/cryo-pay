pub mod infura;

use tokio::time::sleep;
use std::time::Duration;
use std::future::Future;
use tracing::{error, info};
use infura::{BlockGetter, LogsGetter, Monitor, MonitorFilter, MonitorProvider, PayInvoiceEvent};
use crate::monitor::infura::parse_event;
use crate::utils;

pub async fn run_monitor<F, Fut>(process_event: F) -> Result<(), String>
    where
        F: Fn(PayInvoiceEvent) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    let provider_network_link = "https://optimism-sepolia.infura.io/v3/bf3b7185e9c647cca8a376ccd332ee80";
    let address_str = utils::get_env_var("CONTRACT_ADDRESS")?;
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let delay_between_checks = utils::get_env_var("DELAY_BETWEEN_CHECKS")?
        .parse()
        .map_err(|err| {
            error!("foo: {}", err);
            utils::make_err(Box::new(err), "parse DELAY_BETWEEN_CHECKS")
        })?;

    start_monitor(
        provider_network_link,
        &address_str,
        &event_signature,
        delay_between_checks,
        process_event,
    ).await
}

async fn start_monitor<F, Fut>(
    provider_src: &str,
    address_str: &str,
    event_signature: &str,
    delay_between_checks: u64,
    process_event: F,
) -> Result<(), String>
    where
        F: Fn(PayInvoiceEvent) -> Fut,
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
            let result = parse_event(log);

            match result {
                Ok(event) => {
                    if let Err(err) = process_event(event).await {
                        error!("Failed process event: {}", err);
                    }
                }
                Err(err) => error!("Failed decode log: {}", err),
            }
        }

        last_block_number = new_block_number;
    }
}

async fn sleep_duration(to_sleep: u64) {
    info!("Sleeping for {} seconds between logs check", to_sleep);
    sleep(Duration::from_secs(to_sleep)).await;
}
