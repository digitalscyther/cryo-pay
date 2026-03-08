use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Uuid;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
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
