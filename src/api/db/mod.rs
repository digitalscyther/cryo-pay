use serde::Serialize;
use sqlx::{PgPool, types::{chrono::NaiveDateTime, Uuid}};

pub async fn get_db_connection(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(db_url).await
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
}

pub async fn get_invoices(pg_pool: &PgPool) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as!(
        Invoice, "SELECT * FROM invoice ORDER BY created_at DESC"
    )
    .fetch_all(pg_pool)
    .await
}