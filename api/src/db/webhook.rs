use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Uuid;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Clone, Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub secret: String,
    pub created_at: NaiveDateTime,
}

pub async fn create_webhook(pg_pool: &PgPool, url: &str, secret: &str, user_id: &Uuid) -> Result<Webhook, sqlx::Error> {
    sqlx::query_as!(
        Webhook,
        r#"
        INSERT INTO webhook (url, secret, user_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        url,
        secret,
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
