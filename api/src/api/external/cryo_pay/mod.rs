use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use serde::Deserialize;
use tracing::warn;
use uuid::Uuid;
use crate::api::CALLBACK_PATH;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::payments::cryo_pay::{get_paid_payable, PaidPayableResult};
use crate::payments::payable::apply;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(CALLBACK_PATH, get(callback))
        .route("/ping", get(ping_pong))
        .with_state(app_state)
}


#[derive(Deserialize)]
struct PaymentQuery {
    invoice_id: Uuid,
    status: String,
}

async fn callback(
    Query(payment_query): Query<PaymentQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    match payment_query.status == "SUCCESS" {
        true => Ok(match get_paid_payable(&state.db, &payment_query.invoice_id).await {
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
        }),
        false => Err(StatusCode::BAD_REQUEST),
    }
}
