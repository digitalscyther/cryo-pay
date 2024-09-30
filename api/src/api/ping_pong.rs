use axum::Json;
use serde::Serialize;

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