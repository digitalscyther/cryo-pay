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
