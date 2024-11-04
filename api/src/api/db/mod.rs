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
    pub networks: Vec<i32>,
}

pub async fn list_invoices(pg_pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        "SELECT * FROM invoice ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(pg_pool)
    .await
}

pub async fn create_invoice(pg_pool: &PgPool, amount: BigDecimal, seller: &str, networks: &Vec<i32>) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        INSERT INTO invoice (amount, seller, networks)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        amount,
        seller.to_lowercase(),
        networks,
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

pub async fn set_invoice_paid(db: &PgPool, id: Uuid, seller: &str, amount: BigDecimal, buyer: &str, paid_at: NaiveDateTime) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        UPDATE invoice
        SET buyer = $1, paid_at = $2
        WHERE id = $3 AND seller = $4 AND amount = $5
        RETURNING *
        "#,
        buyer.to_lowercase(),
        paid_at,
        id,
        seller.to_lowercase(),
        amount,
    )
        .fetch_one(db)
        .await
}

pub async fn get_block_number(db: &PgPool, network: &str) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT block_number FROM network_monitor WHERE network = $1",
        network
    )
    .fetch_optional(db)
    .await?;

    Ok(result.map(|record| record.block_number))
}

pub async fn create_or_update_block_number(db: &PgPool, network: &str, block_number: i64) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO network_monitor (network, block_number)
        VALUES ($1, $2)
        ON CONFLICT (network) DO UPDATE
        SET block_number = EXCLUDED.block_number
        "#,
        network.to_lowercase(),
        block_number
    )
    .execute(db)
    .await?;

    Ok(())
}
