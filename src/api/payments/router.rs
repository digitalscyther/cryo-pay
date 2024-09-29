use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::routing::{get, post};
use tower_http::trace::TraceLayer;
use crate::api::db::Invoice;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;

pub async fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/invoice", get(get_invoices_handler))
        // .route("/topics/:topic_id/messages", post(create_message_handler))
        // .route("/topics/:topic_id/messages", get(get_messages_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

async fn get_invoices_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Invoice>>, StatusCode> {
    let invoices = state.get_invoices()
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(invoices))
}