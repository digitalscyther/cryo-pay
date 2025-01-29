use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;
use sqlx::error::BoxDynError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub data: Value,
    pub created_at: NaiveDateTime,
    pub paid_at: Option<NaiveDateTime>,
}

pub async fn create_payment(
    pg_pool: &PgPool,
    id: &Uuid,
    data: &Value,
) -> Result<(), sqlx::Error> {
    let data = serde_json::to_value(data)
        .map_err(|err| sqlx::Error::Encode(BoxDynError::from(err)))?;
    let _ = sqlx::query!(
        r#"
        INSERT INTO payments (id, data)
        VALUES ($1, $2)
        "#,
        id,
        data
    )
        .execute(pg_pool)
        .await;

    Ok(())
}

pub async fn get_payment(pg_pool: &PgPool, id: &Uuid) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as!(
        Payment,
        r#"
        SELECT id, data, created_at, paid_at
        FROM payments
        WHERE id = $1
        "#,
        id
    )
        .fetch_optional(pg_pool)
        .await
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct Subscription {
    id: Uuid,
    user_id: Uuid,
    target: String,
    data: Value,
    created_at: NaiveDateTime,
    until: NaiveDateTime,
}

pub async fn create_or_update_subscription(
    pg_pool: &PgPool,
    user_id: &Uuid,
    target: &str,
    data: &Value,
    until: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO subscriptions (user_id, target, data, until)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, target)
        DO UPDATE SET data = EXCLUDED.data, until = EXCLUDED.until
        "#,
        user_id,
        target,
        data,
        until
    )
        .execute(pg_pool)
        .await?;

    Ok(())
}

pub async fn get_active_subscription(
    pg_pool: &PgPool,
    user_id: &Uuid,
    target: &str,
) -> Result<Option<Subscription>, sqlx::Error> {
    sqlx::query_as!(
        Subscription,
        r#"
        SELECT * FROM subscriptions
        WHERE user_id = $1 AND target = $2 AND until > NOW()
        "#,
        user_id,
        target
    )
        .fetch_optional(pg_pool)
        .await
}
