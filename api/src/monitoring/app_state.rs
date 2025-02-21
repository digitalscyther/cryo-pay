use crate::api::state::DB;
use crate::mailer::Mailer;
use crate::telegram::TelegramClient;

#[derive(Clone)]
pub struct MonitorAppState {
    pub db: DB,
    pub telegram_client: TelegramClient,
    pub mailer: Mailer,
}

impl MonitorAppState {
    pub fn new(db: DB, telegram_client: TelegramClient) -> Result<Self, String> {
        let mailer = Mailer::new()?;
        Ok(Self { db, telegram_client, mailer })
    }
}
