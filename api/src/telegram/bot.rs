use tgbot::api::Client;
use tgbot::types::{SetWebhook, Update};
use tgbot::handler::{LongPoll, UpdateHandler, WebhookServer};
use std::net::SocketAddr;
use tracing::info;
use std::str::FromStr;
use crate::utils;

pub enum TelegramBot {
    LongPool,
    Webhook(String)
}

impl TelegramBot {
    pub fn new() -> Result<Self, String> {
        let env_var = utils::get_env_or("TELEGRAM_WEBHOOK_URL", "0".to_string())?;

        Ok(match utils::is_false(&env_var) {
            true => Self::LongPool,
            false => Self::Webhook(env_var),
        })
    }

    pub async fn run(&self, client: &Client) -> Result<(), String> {
        let handler = Handler::new(client.clone());

        match self {
            TelegramBot::LongPool => {
                info!("running telegram bot as LongPool");
                TelegramBot::run_long_pool(handler).await
            },
            TelegramBot::Webhook(url) => {
                client.execute(SetWebhook::new(url))
                    .await
                    .map_err(|err| utils::make_err(Box::new(err), "set webhook"))?;

                info!("running telegram bot as Webhook on {}", url);
                TelegramBot::run_webhook(handler, url).await
            },
        }
    }

    async fn run_long_pool(handler: Handler) -> Result<(), String> {
        LongPoll::new(handler.client.clone(), handler)
            .run()
            .await;

        Ok(())
    }

    async fn run_webhook(handler: Handler, url: &str) -> Result<(), String> {
        WebhookServer::new("/", handler)
            .run(
                SocketAddr::from_str(&url)
                    .map_err(|err| utils::make_err(Box::new(err), "parse socket addr"))?)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "run telegram bot as webhook"))?;

        Ok(())
    }
}

struct Handler {
    client: Client,
}

impl Handler {
    fn new(client: Client) -> Self {
        Self { client }
    }
}

impl UpdateHandler for Handler {
    async fn handle(&self, update: Update) {
        let msg = update.get_message()
            .and_then(|msg| msg.get_text())
            .map(|text| &text.data);

        info!("Got an update with message: {:?}", msg);
    }
}
