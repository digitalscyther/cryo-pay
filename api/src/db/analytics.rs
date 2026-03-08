use bigdecimal::BigDecimal;
use serde::Serialize;
use sqlx::{PgPool, types::{chrono::NaiveDateTime, Uuid}};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct InvoicePeriodStats {
    pub period: NaiveDateTime,
    pub total_invoices: i64,
    pub paid_invoices: i64,
    pub total_amount: BigDecimal,
    pub paid_amount: BigDecimal,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct InvoiceSummary {
    pub total_invoices: i64,
    pub paid_invoices: i64,
    pub total_amount: BigDecimal,
    pub paid_amount: BigDecimal,
}

pub async fn invoice_stats_by_day(
    pg_pool: &PgPool,
    user_id: &Uuid,
    since: NaiveDateTime,
) -> Result<Vec<InvoicePeriodStats>, sqlx::Error> {
    sqlx::query_as!(
        InvoicePeriodStats,
        r#"
        SELECT DATE_TRUNC('day', created_at) AS "period!: NaiveDateTime",
               COUNT(*) AS "total_invoices!: i64",
               COUNT(paid_at) AS "paid_invoices!: i64",
               COALESCE(SUM(amount), 0::NUMERIC) AS "total_amount!: BigDecimal",
               COALESCE(SUM(CASE WHEN paid_at IS NOT NULL THEN amount ELSE 0::NUMERIC END), 0::NUMERIC) AS "paid_amount!: BigDecimal"
        FROM invoice
        WHERE user_id = $1 AND created_at >= $2
        GROUP BY 1 ORDER BY 1 DESC
        "#,
        user_id,
        since,
    )
    .fetch_all(pg_pool)
    .await
}

pub async fn invoice_summary(
    pg_pool: &PgPool,
    user_id: &Uuid,
    since: NaiveDateTime,
) -> Result<InvoiceSummary, sqlx::Error> {
    sqlx::query_as!(
        InvoiceSummary,
        r#"
        SELECT COUNT(*) AS "total_invoices!: i64",
               COUNT(paid_at) AS "paid_invoices!: i64",
               COALESCE(SUM(amount), 0::NUMERIC) AS "total_amount!: BigDecimal",
               COALESCE(SUM(CASE WHEN paid_at IS NOT NULL THEN amount ELSE 0::NUMERIC END), 0::NUMERIC) AS "paid_amount!: BigDecimal"
        FROM invoice
        WHERE user_id = $1 AND created_at >= $2
        "#,
        user_id,
        since,
    )
    .fetch_one(pg_pool)
    .await
}
