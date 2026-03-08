use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Uuid;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Clone, Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
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
