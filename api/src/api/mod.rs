mod ping_pong;
pub mod state;
mod payments;
mod blockchain;
mod auth;
mod middleware;
mod user;
mod external;
mod buy;
pub mod response_error;
pub mod utils;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use axum::http::{header, Method};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use ping_pong::{ping_pong, health_check};
use crate::api::state::DB;
use crate::monitoring::health::DaemonHealth;
use crate::network::Network;
use crate::telegram::TelegramClient;
use crate::utils as base_utils;

const USER_BASE_PATH: &str = "/user";
const PAYMENT_BASE_PATH: &str = "/payment";
const INVOICE_PATH: &str = "/invoice";
const EXTERNAL_BASE_PATH: &str = "/external";
const CRYO_PAY_PATH: &str = "/cryo-pay";
const CALLBACK_PATH: &str = "/callback";

pub fn get_invoice_full_path() -> String {
    base_utils::combine_paths(&[PAYMENT_BASE_PATH, INVOICE_PATH])
}

pub fn get_target_invoice_path(id: &Uuid) -> String {
    base_utils::combine_paths(&[&get_invoice_full_path(), "/", &id.to_string()])
}

pub fn get_cryo_pay_callback_full_path() -> String {
    base_utils::combine_paths(&[EXTERNAL_BASE_PATH, CRYO_PAY_PATH, CALLBACK_PATH])
}


pub async fn run_api(
    networks: Vec<Network>, db: DB, telegram_client: TelegramClient,
    daemon_health: Arc<DaemonHealth>,
) -> Result<(), String> {
    let app_state = state::setup_app_state(networks, db, telegram_client, daemon_health).await?;
    app_state.db.run_migrations()
        .await
        .map_err(|err| base_utils::make_err(Box::new(err), "run migrations"))?;
    let app_state = Arc::new(app_state);

    let health_router = Router::new()
        .route("/health", get(health_check))
        .with_state(app_state.clone());

    let mut router = Router::new()
        .route("/ping", get(ping_pong))
        .merge(health_router)
        .nest("/auth", auth::get_router(app_state.clone()))
        .nest(USER_BASE_PATH, user::get_router(app_state.clone()))
        .nest(PAYMENT_BASE_PATH, payments::get_router(app_state.clone()))
        .nest("/blockchain", blockchain::get_router(app_state.clone()))
        .nest(EXTERNAL_BASE_PATH, external::get_router(app_state.clone()))
        .nest("/buy", buy::get_router(app_state.clone()))
        .layer(TraceLayer::new_for_http());

    let web_origin = base_utils::web_base_url()
        .and_then(|url| url.parse::<header::HeaderValue>()
            .map_err(|e| format!("Invalid WEB_BASE_URL for CORS: {e}")))?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::COOKIE])
        .allow_origin(web_origin)
        .allow_credentials(true);
    router = router.layer(cors);

    let bind_address = base_utils::get_bind_address()?;
    info!("Listening on {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .map_err(|err| base_utils::make_err(Box::new(err), "init listener"))?;

    axum::serve(listener, router.into_make_service()).await
        .map_err(|err| base_utils::make_err(Box::new(err), "start serving"))?;

    Ok(())
}