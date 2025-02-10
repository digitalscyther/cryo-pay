use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::buy::apply_paid_by_id;
use crate::api::CALLBACK_PATH;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;

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
) -> Result<impl IntoResponse, ResponseError> {
    match payment_query.status == "SUCCESS" {
        false => Err(ResponseError::Bad("wrong status")),
        true => apply_paid_by_id(&state, &payment_query.invoice_id)
            .await
            .map(|_| StatusCode::OK),
    }
}
