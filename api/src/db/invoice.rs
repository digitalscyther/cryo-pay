use bigdecimal::BigDecimal;
use sqlx::PgPool;
use sqlx::types::Uuid;
use sqlx::types::chrono::NaiveDateTime;
use super::Invoice;

pub async fn user_own_invoices(
    pg_pool: &PgPool,
    limit: i64,
    offset: i64,
    user_id: &Uuid,
) -> Result<Vec<Invoice>, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        SELECT * FROM invoice
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3"#,
        user_id,
        limit,
        offset,
    )
        .fetch_all(pg_pool)
        .await
}

pub async fn list_invoices(
    pg_pool: &PgPool,
    limit: i64,
    offset: i64,
    user_id: Option<Uuid>,
) -> Result<Vec<Invoice>, sqlx::Error> {
    match user_id {
        None => sqlx::query_as!(
            Invoice,
            r#"
            SELECT * FROM invoice
            WHERE is_private = false
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2"#,
            limit,
            offset,
        )
            .fetch_all(pg_pool)
            .await,
        Some(user_id) => sqlx::query_as!(
            Invoice,
            r#"
            SELECT invoice.* FROM invoice
            WHERE is_private = false OR user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3"#,
            user_id,
            limit,
            offset
        )
            .fetch_all(pg_pool)
            .await
    }
}

pub async fn create_invoice(
    pg_pool: &PgPool,
    amount: BigDecimal,
    seller: &str,
    networks: &Vec<i32>,
    user_id: Option<Uuid>,
    external_id: Option<String>,
    is_private: bool,
) -> Result<Invoice, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        INSERT INTO invoice (amount, seller, networks, user_id, external_id, is_private)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        amount,
        seller.to_lowercase(),
        networks,
        user_id,
        external_id,
        is_private,
    )
        .fetch_one(pg_pool)
        .await
}

pub async fn get_invoice(db: &PgPool, id: &Uuid) -> Result<Option<Invoice>, sqlx::Error> {
    sqlx::query_as!(
        Invoice,
        r#"
        SELECT * FROM invoice
        WHERE id = $1
        "#,
        id
    )
        .fetch_optional(db)
        .await
}

pub async fn get_is_owner(db: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 1 AS some FROM invoice
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
        .fetch_optional(db)
        .await;

    match result {
        Ok(row) => Ok(row.is_some()),
        Err(sqlx::Error::RowNotFound) => Ok(false),
        Err(e) => Err(e),
    }
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

pub async fn delete_own_invoice(db: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM invoice
        WHERE id = $1 AND user_id = $2 AND paid_at IS NULL
        "#,
        id,
        user_id
    )
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}
