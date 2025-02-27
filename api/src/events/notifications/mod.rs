use std::sync::Arc;
use serde_json::json;
use tracing::error;
use uuid::Uuid;
use crate::api::state::DB;
use crate::db::Invoice;
use crate::monitoring::app_state::MonitorAppState;

#[derive(Debug)]
pub enum Notifier {
    Email(EmailNotifier),
    Telegram(TelegramNotifier),
    Webhooks(WebhooksNotifier),
}

#[derive(Debug)]
pub struct EmailNotifier {
    email: String,
}

#[derive(Debug)]
pub struct TelegramNotifier {
    chat_id: String,
}

#[derive(Debug)]
pub struct WebhooksNotifier {
    urls: Vec<String>,
}

impl Notifier {
    pub fn from_email(email: String) -> Self {
        Self::Email(EmailNotifier::new(email))
    }

    pub fn from_telegram_data(chat_id: String) -> Self {
        Self::Telegram(TelegramNotifier::new(chat_id))
    }

    pub fn from_webhooks_urls(urls: Vec<String>) -> Self {
        Self::Webhooks(WebhooksNotifier::new(urls))
    }

    pub async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String> {
        match self {
            Notifier::Email(email) => email.notify(app_state, invoice).await,
            Notifier::Telegram(telegram) => telegram.notify(app_state, invoice).await,
            Notifier::Webhooks(webhooks) => webhooks.notify(app_state, invoice).await,
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

        let webhooks = db.list_webhooks(user_id).await?;
        if !webhooks.is_empty() {
            notifiers.push(
                Notifier::from_webhooks_urls(
                    webhooks.into_iter().map(|wh| wh.url).collect()
                )
            )
        }

        Ok(notifiers)
    }
}

trait Notify {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String>;
}

impl EmailNotifier {
    fn new(email: String) -> Self {
        Self { email }
    }
}

impl TelegramNotifier {
    fn new(chat_id: String) -> Self {
        Self { chat_id }
    }
}

impl WebhooksNotifier {
    fn new(urls: Vec<String>) -> Self {
        Self { urls }
    }
}

impl Notify for EmailNotifier {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String> {
        app_state.mailer.send_invoice_paid(&self.email, &invoice.web_url()?).await
    }
}

impl Notify for TelegramNotifier {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String> {
        app_state.telegram_client.send_invoice_paid(&self.chat_id, &invoice).await
    }
}

impl Notify for WebhooksNotifier {
    async fn notify(&self, app_state: Arc<MonitorAppState>, invoice: Invoice) -> Result<(), String> {
        let payload = json!({
            "invoice_id": invoice.id,
            "paid_at": invoice.paid_at,
        });

        for url in &self.urls {
            if let Err(err) = app_state.webhooker.send(url, &payload).await {
                error!(err)
            }
        }

        Ok(())
    }
}
