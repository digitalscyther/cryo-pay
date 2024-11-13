use tgbot::api::Client;
use bot::TelegramBot;

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

    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), String> {
        client::send_message(&self.client, chat_id, text).await
    }

    pub async fn get_bot_name(&self) -> Result<String, String> {
        client::get_bot_name(&self.client).await
    }

    pub async fn run_as_bot(&self) -> Result<(), String> {
        let telegram_bot = TelegramBot::new()?;

        telegram_bot.run(&self.client).await
    }
}
