mod subscription;
mod to_pay;
mod donation;

use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::{Extension, Json, middleware, Router};
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;
use crate::api::external::cryo_pay::apply_paid_by_id;
use crate::api::middleware::{extract_user, only_auth};
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::api::utils::Pagination;
use crate::db::billing::Payment;
use crate::db::User;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/payment", get(list_payment))
        .layer(middleware::from_fn_with_state(app_state.clone(), only_auth))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .route("/payment/:payment_id/recheck", post(recheck))
        .route("/payment/:payment_id", get(get_payment))
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

async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<PaymentResponse>, ResponseError> {
    state.db.get_payment(&payment_id)
        .await
        .map_err(ResponseError::from_error)?
        .ok_or(ResponseError::NotFound)
        .map(|p| Json(p.into()))
}

async fn list_payment(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<PaymentResponse>>, ResponseError> {
    let (limit, offset) = pagination.get_valid(10)?;

    Ok(Json(state.db.user_list_payment(&user.id, limit, offset)
        .await
        .map_err(ResponseError::from_error)?
        .into_iter()
        .map(|p| p.into())
        .collect::<Vec<PaymentResponse>>()))
}
