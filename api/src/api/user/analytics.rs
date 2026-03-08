use std::sync::Arc;
use axum::{Extension, Json, extract::{Query, State}};
use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::{User, analytics::{InvoicePeriodStats, InvoiceSummary}};

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 { 30 }

#[derive(Serialize, utoipa::ToSchema)]
pub struct AnalyticsResponse {
    pub summary: InvoiceSummary,
    pub by_day: Vec<InvoicePeriodStats>,
    pub period_days: u32,
}

#[utoipa::path(
    get,
    path = "/user/analytics",
    params(("days" = Option<u32>, Query, description = "Number of days to look back (default 30, max 365)")),
    responses(
        (status = 200, description = "Invoice analytics for the authenticated seller", body = AnalyticsResponse),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user"
)]
pub(crate) async fn get_analytics(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse>, ResponseError> {
    let days = query.days.min(365).max(1);
    let since: NaiveDateTime = (Utc::now() - Duration::days(days as i64)).naive_utc();

    let summary = state.db.invoice_summary(&user.id, since)
        .await
        .map_err(ResponseError::from)?;

    let by_day = state.db.invoice_stats_by_day(&user.id, since)
        .await
        .map_err(ResponseError::from)?;

    Ok(Json(AnalyticsResponse { summary, by_day, period_days: days }))
}
