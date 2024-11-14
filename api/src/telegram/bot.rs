use tgbot::api::Client;
use tgbot::types::{SetWebhook, Update};
use tgbot::handler::{LongPoll, UpdateHandler, WebhookServer};
use std::net::SocketAddr;
use tracing::{error, info, warn};
use std::str::FromStr;
use uuid::Uuid;
use crate::api::state::DB;
use crate::telegram::client::send_message;
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

    pub async fn run(&self, client: &Client, db: DB) -> Result<(), String> {
        let handler = Handler::new(client.clone(), db);

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
    db: DB,
}

impl Handler {
    fn new(client: Client, db: DB) -> Self {
        Self { client, db }
    }
}

impl UpdateHandler for Handler {
    async fn handle(&self, update: Update) {
        let chat_id = match update.get_chat_id() {
            None => {
                warn!("chat id not extractable: {:?}", update);
                return;
            }
            Some(chat_id) => chat_id
        };

        let user_id = match update.get_message()
            .and_then(|msg| msg.get_text())
            .map(|text| &text.data) {
            None => return,
            Some(text) => {
                let texts = text.split(" ").collect::<Vec<&str>>();
                let user_id_str = match texts.last() {
                    None => return,
                    Some(text) => text.to_owned()
                };
                match Uuid::parse_str(user_id_str) {
                    Ok(user_id) => user_id,
                    Err(_) => return,
                }
            }
        };

        match self.db.set_user_telegram_chat_id(&user_id, Some(chat_id.to_string())).await {
            Ok(_) => info!("Set telegram_chat_id for user_id={:?}", user_id),
            Err(err) => error!(err),
        };

        if let Err(err) = send_message(
            &self.client,
            chat_id.into(),
            "We have saved your contact and will be able to send you payment notifications here."
        ).await { error!(err) }
    }
}
