pub mod analytics;
pub mod billing;
pub mod invoice;
pub mod blockchain;
pub mod user;
pub mod api_key;
pub mod callback_url;
pub mod webhook;

// Re-export structs so existing imports (use crate::db::{ApiKey, CallbackUrl, Invoice, User, Webhook}) keep working
pub use api_key::ApiKey;
pub use callback_url::CallbackUrl;
pub use webhook::Webhook;

use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{PgPool, types::{chrono::NaiveDateTime, Uuid}};
use crate::utils;

pub async fn get_db_connection(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(db_url).await
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct User {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub firebase_user_id: String,
    pub email: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub email_notification: bool,
    pub telegram_notification: bool,
}

#[derive(Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Invoice {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    #[schema(value_type = String, example = "10.00")]
    pub amount: BigDecimal,
    pub seller: String,
    pub buyer: Option<String>,
    pub paid_at: Option<NaiveDateTime>,
    pub networks: Vec<i32>,
    pub user_id: Option<Uuid>,
    pub external_id: Option<String>,
    is_private: bool,
}

impl Invoice {
    pub fn web_url(&self) -> Result<String, String> {
        utils::get_invoice_url(&self.id)
    }
}
