use tokio::time::sleep;
use std::time::Duration;
use ethers::prelude::{Filter, Log, Provider, ProviderError, U64};
use std::future::Future;
use ethers::addressbook::Address;
use ethers::middleware::Middleware;
use ethers::providers::Http;

use crate::utils;

pub async fn run_monitor<F, Fut>(process_log: F) -> Result<(), String>
where
    F: Fn(Log) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let provider_network_link = "https://optimism-sepolia.infura.io/v3/bf3b7185e9c647cca8a376ccd332ee80";
    let address_str = utils::get_env_var("CONTRACT_ADDRESS")?;
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let delay_between_checks = 30;

    start_monitor(
        provider_network_link,
        &address_str,
        &event_signature,
        delay_between_checks,
        process_log
    ).await
}

async fn start_monitor<F, Fut>(
    provider_src: &str,
    address_str: &str,
    event_signature: &str,
    delay_between_checks: u64,
    process_log: F
) -> Result<(), String>
where
    F: Fn(Log) -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    let provider = Provider::<Http>::try_from(provider_src)
        .map_err(|err| utils::make_err(Box::new(err), "create provider"))?;

    let address = address_str.parse::<Address>()
        .map_err(|err| utils::make_err(Box::new(err), "parse address"))?;

    let base_filter = Filter::new()
        .address(address)
        .event(event_signature);

    let mut last_block_number = get_block_number(&provider).await
        .map_err(|err| utils::make_err(Box::new(err), "get start block number"))?;

    loop {
        sleep_duration(delay_between_checks).await;

        let new_block_number = get_block_number(&provider).await
            .map_err(|err| utils::make_err(Box::new(err), "get start block number"))?;

        if new_block_number <= last_block_number {
            continue;
        }

        let logs = get_logs(
            &provider, base_filter.clone(), last_block_number, new_block_number,
        ).await?;

        for log in logs {
            if let Err(err) = process_log(log).await {
                println!("Failed process_log: {:?}", err);
            }
        }

        last_block_number = new_block_number;
    }
}

async fn get_block_number(provider: &Provider<Http>) -> Result<U64, ProviderError> {
    provider.get_block_number().await
}

async fn sleep_duration(to_sleep: u64) {
    println!("Sleeping for {} seconds", to_sleep);
    sleep(Duration::from_secs(to_sleep)).await;
}

async fn get_logs(provider: &Provider<Http>, base_filter: Filter, block_from: U64, block_to: U64) -> Result<Vec<Log>, String> {
    let filter = base_filter.clone()
        .from_block(block_from)
        .to_block(block_to);

    provider.get_logs(&filter).await
        .map_err(|err| utils::make_err(Box::new(err), "get logs"))
}
