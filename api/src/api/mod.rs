mod ping_pong;
pub mod state;
mod db;
mod payments;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};

use ping_pong::ping_pong;
use crate::utils;

pub async fn run_api() -> Result<(), String> {
    tracing_subscriber::fmt().json()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = state::setup_app_state().await.expect("Failed to build AppState");
    app_state.db.run_migrations()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "run migrations"))?;
    let app_state = Arc::new(app_state);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = Router::new()
        .route("/ping", get(ping_pong))
        .nest("/payment", payments::router::get_router(app_state.clone()).await)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let host = utils::get_env_var("HOST")?;
    let port = utils::get_env_var("PORT")?;
    let bind_address = format!("{}:{}", host, port);
    info!("Listening on {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Failed init listener");

    axum::serve(listener, router.into_make_service()).await.expect("Failed start serving");

    Ok(())
}