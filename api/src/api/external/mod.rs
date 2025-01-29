use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use crate::api::CRYO_PAY_PATH;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;

mod cryo_pay;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .nest(CRYO_PAY_PATH, cryo_pay::get_router(app_state.clone()))
}