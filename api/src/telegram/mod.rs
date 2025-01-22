use tgbot::api::Client;
use tgbot::types::{ChatId, Integer};
use bot::TelegramBot;
use crate::api::state::DB;
use crate::db::Invoice;
use crate::utils;

mod client;
mod bot;

#[derive(Clone)]
pub struct TelegramClient {
    client: Client,
}

impl TelegramClient {
    pub async fn new() -> Result<Self, String> {
        let client = client::get_client().await?;

        Ok(Self { client })
    }

    pub async fn send_invoice_paid(&self, chat_id: &str, invoice: &Invoice) -> Result<(), String> {
        let chat_id: ChatId = chat_id
            .parse::<Integer>()
            .map_err(|err| utils::make_err(Box::new(err), "parse chat id"))?
            .into();

        client::send_invoice_paid(&self.client, chat_id, invoice).await
    }

    pub async fn get_bot_name(&self) -> Result<String, String> {
        client::get_bot_name(&self.client).await
    }

    pub async fn run_as_bot(&self, db: DB) -> Result<(), String> {
        let telegram_bot = TelegramBot::new()?;

        telegram_bot.run(&self.client, db).await
    }
}
