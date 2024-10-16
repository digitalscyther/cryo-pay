use tokio::time::sleep;
use std::time::Duration;
use ethers::prelude::{Filter, Log, Provider, U128, U64};
use std::future::Future;
use ethers::addressbook::Address;
use ethers::contract::EthEvent;
use ethers::middleware::Middleware;
use ethers::providers::Http;
use tracing::{error, info};

use crate::utils;

pub async fn run_monitor<F, Fut>(process_event: F) -> Result<(), String>
    where
        F: Fn(PayInvoiceEvent) -> Fut,
        Fut: Future<Output=Result<(), String>>,
{
    let provider_network_link = "https://optimism-sepolia.infura.io/v3/bf3b7185e9c647cca8a376ccd332ee80";
    let address_str = utils::get_env_var("INVOICE_ADDRESS")?;
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

struct Monitor {
    provider: Provider<Http>,
    base_filter: Filter,
}

enum MonitorProvider<'a> {
    Url(&'a str),
    Provider(Provider<Http>),
}

enum MonitorFilter<'a> {
    Signature(&'a str),
    Filter(Filter),
}

impl Monitor {
    fn new(provider: MonitorProvider, filter: MonitorFilter) -> Result<Self, String> {
        let provider = match provider {
            MonitorProvider::Url(provider_url) => Provider::<Http>::try_from(provider_url)
                .map_err(|err| utils::make_err(Box::new(err), "create provider"))?,
            MonitorProvider::Provider(provider) => provider
        };

        let base_filter = match filter {
            MonitorFilter::Signature(signature) => Filter::new().event(&signature),
            MonitorFilter::Filter(filter) => filter
        };

        Ok(Self { provider, base_filter })
    }

    fn with_address(self, address: &str) -> Result<Self, String> {
        let provider = self.provider;
        let base_filter = self.base_filter
            .address(address.parse::<Address>()
                .map_err(|err| utils::make_err(Box::new(err), "parse address"))?);

        Self::new(MonitorProvider::Provider(provider), MonitorFilter::Filter(base_filter))
    }
}

trait LogsGetter {
    async fn get_logs(&self, block_from: U64, block_to: U64) -> Result<Vec<Log>, String>;
}

impl LogsGetter for Monitor {
    async fn get_logs(&self, block_from: U64, block_to: U64) -> Result<Vec<Log>, String> {
        let filter = self.base_filter.clone()
            .from_block(block_from)
            .to_block(block_to);

        self.provider.get_logs(&filter).await
            .map_err(|err| utils::make_err(Box::new(err), "get logs"))
    }
}

trait BlockGetter {
    async fn get_block_number(&self) -> Result<U64, String>;
}

impl BlockGetter for Monitor {
    async fn get_block_number(&self) -> Result<U64, String> {
        self.provider.get_block_number().await
            .map_err(|err| utils::make_err(Box::new(err), "get start block number"))
    }
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
            let result = <PayInvoiceEvent as EthEvent>::decode_log(&log.into())
                .map_err(|err| utils::make_err(Box::new(err), "decode log"));

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
