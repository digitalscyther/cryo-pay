mod ping_pong;
pub mod state;
mod payments;
mod blockchain;
mod auth;
mod middleware;
mod user;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use tower_http::cors::{CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use ping_pong::ping_pong;
use crate::api::state::DB;
use crate::network::Network;
use crate::telegram::TelegramClient;
use crate::utils;

const USER_BASE_PATH: &str = "/user";
const PAYMENT_BASE_PATH: &str = "/payment";
const INVOICE_PATH: &str = "/invoice";

pub fn get_invoice_full_path() -> String {
    format!("{}{}", PAYMENT_BASE_PATH, INVOICE_PATH)
}

pub async fn run_api(networks: Vec<Network>, db: DB, telegram_client: TelegramClient) -> Result<(), String> {
    let app_state = state::setup_app_state(networks, db, telegram_client).await?;
    app_state.db.run_migrations()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "run migrations"))?;
    let app_state = Arc::new(app_state);

    let mut router = Router::new()
        .route("/ping", get(ping_pong))
        .nest("/auth", auth::get_router(app_state.clone()))
        .nest(USER_BASE_PATH, user::get_router(app_state.clone()))
        .nest(PAYMENT_BASE_PATH, payments::get_router(app_state.clone()))
        .nest("/blockchain", blockchain::get_router(app_state.clone()))
        .layer(TraceLayer::new_for_http());

    if Some("1") == utils::get_env_or("DEBUG", "0".to_string()).ok().as_deref() {
        info!("will be allowed any cors");
        let cors = CorsLayer::very_permissive();
        router = router.layer(cors);
    }

    let bind_address = utils::get_bind_address()?;
    info!("Listening on {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .map_err(|err| utils::make_err(Box::new(err), "init listener"))?;

    axum::serve(listener, router.into_make_service()).await
        .map_err(|err| utils::make_err(Box::new(err), "start serving"))?;

    Ok(())
}