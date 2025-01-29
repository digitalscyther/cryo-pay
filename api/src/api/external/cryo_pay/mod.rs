use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use crate::api::CALLBACK_PATH;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::payments::cryo_pay::CryoPayApi;

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
    let cryo_pay_api = CryoPayApi::default();

    Ok(match payment_query.status == "SUCCESS" && cryo_pay_api.is_invoice_paid(&payment_query.invoice_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        false => StatusCode::BAD_REQUEST,
        true => match state.db.get_payment(&payment_query.invoice_id)
            .await {
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Ok(None) => StatusCode::NOT_FOUND,
            Ok(Some(payment)) => {
                info!("Payment #{} type={} got!", payment.id, payment.data);
                StatusCode::OK
            }
        }
    })
}