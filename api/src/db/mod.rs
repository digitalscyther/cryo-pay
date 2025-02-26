pub mod billing;

use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{PgPool, types::{chrono::NaiveDateTime, Uuid}};
use crate::utils;

pub async fn get_db_connection(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(db_url).await
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub firebase_user_id: String,
    pub email: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub email_notification: bool,
    pub telegram_notification: bool,
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
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

pub async fn get_user_by_id(db: &PgPool, id: &Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT * FROM "users"
        WHERE id = $1
        "#,
        id
    )
        .fetch_one(db)
        .await
}

pub async fn get_or_create_user(db: &PgPool, firebase_user_id: &str, email: Option<String>) -> Result<User, sqlx::Error> {
    let existing_user = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM "users"
        WHERE firebase_user_id = $1 AND email = $2
        "#,
        firebase_user_id,
        email
    )
        .fetch_optional(db)
        .await?;

    if let Some(user) = existing_user {
        return Ok(user);
    }

    let new_user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO "users" (firebase_user_id, email)
        VALUES ($1, $2)
        ON CONFLICT (firebase_user_id)
        DO UPDATE SET email = EXCLUDED.email
        RETURNING *
        "#,
        firebase_user_id,
        email
    )
        .fetch_one(db)
        .await?;

    Ok(new_user)
}

pub async fn update_user(
    db: &PgPool,
    user_id: &Uuid,
    email_notification: Option<bool>,
    telegram_notification: Option<bool>,
) -> Result<User, sqlx::Error> {
    if let Some(email_notification) = email_notification {
        if let Some(telegram_notification) = telegram_notification {
            return sqlx::query_as!(
                User,
                r#"
                UPDATE "users"
                SET email_notification = $1, telegram_notification = $2
                WHERE id = $3
                RETURNING *
                "#,
                email_notification,
                telegram_notification,
                user_id,
            )
                .fetch_one(db)
                .await;
        }

        return sqlx::query_as!(
            User,
            r#"
            UPDATE "users"
            SET email_notification = $1
            WHERE id = $2
            RETURNING *
            "#,
            email_notification,
            user_id,
        )
            .fetch_one(db)
            .await;
    }

    if let Some(telegram_notification) = telegram_notification {
        return sqlx::query_as!(
            User,
            r#"
            UPDATE "users"
            SET telegram_notification = $1
            WHERE id = $2
            RETURNING *
            "#,
            telegram_notification,
            user_id,
        )
            .fetch_one(db)
            .await;
    }

    get_user_by_id(db, user_id).await
}

pub async fn set_user_telegram_chat_id(db: &PgPool, id: &Uuid, telegram_chat_id: Option<String>) -> Result<(), sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE "users"
        SET telegram_chat_id = $1
        WHERE id = $2
        "#,
        telegram_chat_id,
        id,
    )
        .execute(db)
        .await?;

    Ok(())
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

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key: String,
    pub created: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
}

pub async fn delete_api_key(pg_pool: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM api_key
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
        .execute(pg_pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_api_key(
    pg_pool: &PgPool,
    user_id: &Uuid,
    hashed_api_key: &str,
) -> Result<ApiKey, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"
        INSERT INTO api_key (user_id, api_key)
        VALUES ($1, $2)
        RETURNING *
        "#,
        user_id,
        hashed_api_key
    )
        .fetch_one(pg_pool)
        .await
}

pub async fn get_api_key(pg_pool: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<Option<ApiKey>, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"
        SELECT * FROM api_key
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
        .fetch_optional(pg_pool)
        .await
}

pub async fn get_api_key_by_api_key(pg_pool: &PgPool, api_key: &str) -> Result<Option<ApiKey>, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"
        SELECT * FROM api_key
        WHERE api_key = $1
        "#,
        api_key
    )
        .fetch_optional(pg_pool)
        .await
}

pub async fn list_api_key(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Vec<ApiKey>, sqlx::Error> {
    sqlx::query_as!(
        ApiKey,
        r#"
        SELECT * FROM api_key
        WHERE user_id = $1
        ORDER BY created DESC
        "#,
        user_id
    )
        .fetch_all(pg_pool)
        .await
}

pub async fn update_api_key_last_used(pg_pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE api_key
        SET last_used = NOW()
        WHERE id = $1
        "#,
        id
    )
        .execute(pg_pool)
        .await?;
    Ok(())
}

pub async fn count_api_keys_by_user_id(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM api_key
        WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_one(pg_pool)
        .await
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct CallbackUrl {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
}


pub async fn create_callback_url(pg_pool: &PgPool, url: &str, user_id: &Uuid) -> Result<CallbackUrl, sqlx::Error> {
    sqlx::query_as!(
        CallbackUrl,
        r#"
        INSERT INTO callback_urls (url, user_id)
        VALUES ($1, $2)
        RETURNING *
        "#,
        url,
        user_id
    )
        .fetch_one(pg_pool)
        .await
}

pub async fn list_callback_urls_by_user_id(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Vec<CallbackUrl>, sqlx::Error> {
    sqlx::query_as!(
        CallbackUrl,
        r#"
        SELECT * FROM callback_urls
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
        .fetch_all(pg_pool)
        .await
}

pub async fn delete_callback_url_by_id_and_user_id(
    pg_pool: &PgPool,
    id: &Uuid,
    user_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM callback_urls
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
        .execute(pg_pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn count_callback_urls_by_user_id(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM callback_urls
        WHERE user_id = $1
        "#,
        user_id
    )
        .fetch_one(pg_pool)
        .await
}

pub async fn exists_callback_url(pg_pool: &PgPool, url: &str, user_id: &Uuid) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM callback_urls WHERE user_id = $1
        ) AS exists
        "#,
        user_id
    )
        .fetch_one(pg_pool)
        .await?
        .exists {
        Some(exists) if !exists => return Ok(true),
        _ => {}
    }

    match sqlx::query!(
        r#"
        SELECT 1 AS some FROM callback_urls
        WHERE url = $1 AND user_id = $2
        "#,
        url,
        user_id
    )
        .fetch_optional(pg_pool)
        .await {
        Ok(row) => Ok(row.is_some()),
        Err(sqlx::Error::RowNotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
}

pub async fn create_webhook(pg_pool: &PgPool, url: &str, user_id: &Uuid) -> Result<Webhook, sqlx::Error> {
    sqlx::query_as!(
        Webhook,
        r#"
        INSERT INTO webhook (url, user_id)
        VALUES ($1, $2)
        RETURNING *
        "#,
        url,
        user_id
    )
    .fetch_one(pg_pool)
    .await
}

pub async fn list_webhooks_by_user_id(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Vec<Webhook>, sqlx::Error> {
    sqlx::query_as!(
        Webhook,
        r#"
        SELECT * FROM webhook
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pg_pool)
    .await
}

pub async fn delete_webhook_by_id_and_user_id(
    pg_pool: &PgPool,
    id: &Uuid,
    user_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM webhook
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        user_id
    )
    .execute(pg_pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn count_webhooks_by_user_id(
    pg_pool: &PgPool,
    user_id: &Uuid,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM webhook
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pg_pool)
    .await
}
