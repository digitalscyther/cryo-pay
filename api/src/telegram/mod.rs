use tgbot::api::Client;

mod client;

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
}