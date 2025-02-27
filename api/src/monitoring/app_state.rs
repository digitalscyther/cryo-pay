use std::time::Duration;
use reqwest::Client;
use serde_json::Value;
use crate::api::state::DB;
use crate::mailer::Mailer;
use crate::telegram::TelegramClient;
use crate::utils;

#[derive(Clone)]
pub struct MonitorAppState {
    pub db: DB,
    pub telegram_client: TelegramClient,
    pub mailer: Mailer,
    pub webhooker: Webhooker,
}

impl MonitorAppState {
    pub fn new(db: DB, telegram_client: TelegramClient) -> Result<Self, String> {
        let mailer = Mailer::new()?;
        let webhooker = Webhooker { };
        Ok(Self { db, telegram_client, mailer, webhooker })
    }
}

#[derive(Clone)]
pub struct Webhooker;

impl Webhooker {
    pub async fn send(&self, url: &str, payload: &Value) -> Result<(), String> {
        let status = Client::new()
            .post(url)
            .timeout(Duration::from_secs(30))
            .json(payload)
            .send()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "send to webhook"))?
            .status();

        if !status.is_success() {
            return Err(format!("Webhook to {} failed with status: {}", url, status));
        }

        Ok(())
    }
}
