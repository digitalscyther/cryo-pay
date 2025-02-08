mod subscription;
mod to_pay;
mod donation;

use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::{get, post};
use tracing::warn;
use uuid::Uuid;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::payments::cryo_pay::{get_paid_payable, PaidPayableResult};
use crate::payments::payable::apply;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/recheck/:payment_id", post(recheck))
        .with_state(app_state.clone())
        .route("/ping", get(ping_pong))
        .nest("/subscription", subscription::get_router(app_state.clone()))
        .nest("/donation", donation::get_router(app_state.clone()))
}

async fn recheck(
    State(state): State<Arc<AppState>>,
    Path(payment_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(match get_paid_payable(&state.db, &payment_id).await {
        Err(err) => {
            warn!("{err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(PaidPayableResult::NotPaid) => StatusCode::BAD_REQUEST,
        Ok(PaidPayableResult::NotFound) => StatusCode::NOT_FOUND,
        Ok(PaidPayableResult::Payment(payment)) => match apply(&state, payment).await {
            Err(err) => {
                warn!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Ok(_) => StatusCode::OK
        },
    })
}