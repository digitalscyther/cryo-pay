use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{PgPool, types::{chrono::NaiveDateTime, Uuid}};

pub async fn get_db_connection(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(db_url).await
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub amount: BigDecimal,
    pub seller: String,
    pub buyer: Option<String>,
    pub paid_at: Option<NaiveDateTime>,
}

pub async fn list_invoices(pg_pool: &PgPool) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as!(
        Invoice, "SELECT * FROM invoice ORDER BY created_at DESC"
    )
    .fetch_all(pg_pool)
    .await
}

pub async fn create_invoice(pg_pool: &PgPool, amount: BigDecimal, seller: &str) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        INSERT INTO invoice (amount, seller)
        VALUES ($1, $2)
        RETURNING *
        "#,
        amount,
        seller
    )
    .fetch_one(pg_pool)
    .await
}

pub async fn get_invoice(db: &PgPool, id: Uuid) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        SELECT * FROM invoice
        WHERE id = $1
        "#,
        id
    )
        .fetch_one(db)
        .await
}

pub async fn set_invoice_paid(db: &PgPool, id: Uuid, buyer: &str, paid_at: NaiveDateTime) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        UPDATE invoice
        SET buyer = $1, paid_at = $2
        WHERE id = $3
        RETURNING *
        "#,
        buyer,
        paid_at,
        id
    )
        .fetch_one(db)
        .await
}
