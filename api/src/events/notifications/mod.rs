use std::sync::Arc;
use uuid::Uuid;
use crate::api::state::DB;
use crate::db::Invoice;
use crate::monitor::MonitorAppState;

#[derive(Debug)]
pub enum Notifier {
    Email(EmailNotifier),
    Telegram(TelegramNotifier)
}

#[derive(Debug)]
pub struct EmailNotifier {
    email: String
}

#[derive(Debug)]
pub struct TelegramNotifier {
    chat_id: String
}

impl Notifier {
    pub fn from_email(email: String) -> Self {
        Self::Email(EmailNotifier::new(email))
    }

    pub fn from_telegram_data(chat_id: String) -> Self {
        Self::Telegram(TelegramNotifier::new(chat_id))
    }

    pub async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String> {
        match self {
            Notifier::Email(email) => email.notify(app_state, invoice).await,
            Notifier::Telegram(telegram) => telegram.notify(app_state, invoice).await,
        }
    }

    pub async fn get_notifiers(db: &DB, user_id: &Uuid) -> Result<Vec<Notifier>, String> {
        let mut notifiers = vec![];

        let user = db.get_user_by_id(user_id).await?;

        if user.email_notification {
            if let Some(email) = user.email {
                notifiers.push(Notifier::from_email(email))
            }
        }

        if user.telegram_notification {
            if let Some(chat_id) = user.telegram_chat_id {
                notifiers.push(Notifier::from_telegram_data(chat_id))
            }
        }

        Ok(notifiers)
    }
}

trait Notify {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String>;
}

impl EmailNotifier {
    fn new(email: String) -> Self {
        Self{ email }
    }
}

impl TelegramNotifier {
    fn new(chat_id: String) -> Self {
        Self{ chat_id }
    }
}

impl Notify for EmailNotifier {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) ->  Result<(), String>{
        app_state.mailer.send_invoice_paid(&self.email, &invoice.web_url()?).await
    }
}

impl Notify for TelegramNotifier {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) ->  Result<(), String>{
        app_state.telegram_client.send_invoice_paid(&self.chat_id, &invoice.web_url()?).await
    }
}
