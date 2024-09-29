use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use uuid::Uuid;
use crate::api::db::Invoice;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;

pub async fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/invoice", get(get_invoices_handler))
        .route("/invoice", post(create_invoice_handler))
        .route("/invoice/:invoice_id", get(get_invoice_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

async fn get_invoices_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Invoice>>, StatusCode> {
    let invoices = state.db.list_invoices()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(invoices))
}

#[derive(Deserialize)]
struct CreateInvoiceRequest {
    amount: BigDecimal,
    seller: String,
}

async fn create_invoice_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Json<Invoice>, StatusCode> {
    let invoice = state.db.create_invoice(payload.amount, &payload.seller)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(invoice))
}

async fn get_invoice_handler(
    Path(invoice_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let invoice = state.db.get_invoice(invoice_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(invoice))
}