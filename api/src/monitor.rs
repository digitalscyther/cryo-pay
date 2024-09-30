use tokio::time::sleep;
use std::time::Duration;
use ethers::prelude::{Filter, Log, Provider, ProviderError, U128, U64};
use std::future::Future;
use ethers::addressbook::Address;
use ethers::contract::EthEvent;
use ethers::middleware::Middleware;
use ethers::providers::Http;
use tracing::error;

use crate::utils;

pub async fn run_monitor<F, Fut>(process_event: F) -> Result<(), String>
    where
        F: Fn(PayInvoiceEvent) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    let provider_network_link = "https://optimism-sepolia.infura.io/v3/bf3b7185e9c647cca8a376ccd332ee80";
    let address_str = utils::get_env_var("CONTRACT_ADDRESS")?;
    let event_signature = utils::get_env_var("EVENT_SIGNATURE")?;
    let delay_between_checks = 5 * 60;

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
            let result = <PayInvoiceEvent as EthEvent>::decode_log(&log.into())
                .map_err(|err| utils::make_err(Box::new(err), "decode log"));

            match result {
                Ok(event) => {
                    if let Err(err) = process_event(event).await {
                        error!("{}", err);
                    }
                }
                Err(err) => error!("{}", err),
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

#[derive(Debug, Clone, EthEvent)]
pub struct PayInvoiceEvent {
    pub invoice_id: String,
    #[ethevent(indexed)]
    pub seller: Address,
    #[ethevent(indexed)]
    pub payer: Address,
    pub paid_at: U128,
    pub amount: U128,
}
