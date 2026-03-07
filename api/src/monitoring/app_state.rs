use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
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
        let webhooker = Webhooker;
        Ok(Self { db, telegram_client, mailer, webhooker })
    }
}

#[derive(Clone)]
pub struct Webhooker;

impl Webhooker {
    pub async fn send(&self, url: &str, secret: &str, payload: &Value) -> Result<(), String> {
        let body = serde_json::to_string(payload)
            .map_err(|err| utils::make_err(Box::new(err), "serialize webhook payload"))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut request = Client::new()
            .post(url)
            .timeout(Duration::from_secs(30))
            .header("content-type", "application/json")
            .header("X-Webhook-Timestamp", timestamp.to_string());

        // Empty secret = legacy webhook, no signature (backwards compatible)
        if !secret.is_empty() {
            let signed_payload = format!("{timestamp}.{body}");
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|err| utils::make_err(Box::new(err), "create HMAC"))?;
            mac.update(signed_payload.as_bytes());
            let signature = hex::encode(mac.finalize().into_bytes());
            request = request.header("X-Signature-256", signature);
        }

        let status = request
            .body(body)
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
