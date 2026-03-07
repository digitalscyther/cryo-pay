use std::sync::Arc;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use crate::api::state::AppState;

#[derive(Serialize)]
pub struct PongResponse {
    message: String,
}

pub async fn ping_pong() -> Json<PongResponse> {
    let response = PongResponse {
        message: "pong".to_string(),
    };
    Json(response)
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    postgres: bool,
    redis: bool,
    daemon: bool,
}

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (pg_result, redis_result) = tokio::join!(
        state.db.health_check(),
        state.redis.health_check(),
    );
    let postgres = pg_result.is_ok();
    let redis = redis_result.is_ok();
    let daemon = state.daemon_health.is_healthy();

    let all_healthy = postgres && redis && daemon;
    let status = if all_healthy { "ok" } else { "degraded" }.to_string();
    let code = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    (code, Json(HealthResponse { status, postgres, redis, daemon }))
}
