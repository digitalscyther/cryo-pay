mod subscription;
mod to_pay;
mod donation;

use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;
use crate::api::external::cryo_pay::apply_paid_by_id;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::billing::Payment;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/recheck/:payment_id", post(recheck))
        .with_state(app_state.clone())
        .route("/ping", get(ping_pong))
        .nest("/subscription", subscription::get_router(app_state.clone()))
        .nest("/donation", donation::get_router(app_state.clone()))
}

#[derive(Serialize)]
struct PaymentResponse {
    id: Uuid,
    data: Value,
    created_at: NaiveDateTime,
    paid_at: Option<NaiveDateTime>,
}


impl From<Payment> for PaymentResponse {
    fn from(value: Payment) -> Self {
        Self {
            id: value.id,
            data: value.data,
            created_at: value.created_at,
            paid_at: value.paid_at
        }
    }
}

async fn recheck(
    State(state): State<Arc<AppState>>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<PaymentResponse>, ResponseError> {
    apply_paid_by_id(&state, &payment_id)
        .await
        .map(|p| Json(p.into()))
}
