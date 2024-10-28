use ethers::{
    addressbook::Address,
    middleware::Middleware,
    prelude::{Filter, Log, Provider, U64},
    providers::Http
};
use crate::utils;

pub struct Monitor {
    provider: Provider<Http>,
    base_filter: Filter,
}

pub enum MonitorProvider<'a> {
    Url(&'a str),
    Provider(Provider<Http>),
}

pub enum MonitorFilter<'a> {
    Signature(&'a str),
    Filter(Filter),
}

impl Monitor {
    pub fn new(provider: MonitorProvider, filter: MonitorFilter) -> Result<Self, String> {
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

    pub fn with_address(self, address: &str) -> Result<Self, String> {
        let provider = self.provider;
        let base_filter = self.base_filter
            .address(address.parse::<Address>()
                .map_err(|err| utils::make_err(Box::new(err), "parse address"))?);

        Self::new(MonitorProvider::Provider(provider), MonitorFilter::Filter(base_filter))
    }
}

pub trait LogsGetter {
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

pub trait BlockGetter {
    async fn get_block_number(&self) -> Result<U64, String>;
}

impl BlockGetter for Monitor {
    async fn get_block_number(&self) -> Result<U64, String> {
        self.provider.get_block_number().await
            .map_err(|err| utils::make_err(Box::new(err), "get block number"))
    }
}
