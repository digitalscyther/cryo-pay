use axum::Json;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde_json::{json, Value};
use tracing::error;

#[derive(Debug, Clone)]
pub enum ResponseError {
    Bad(String),
    InternalServerError(String),
    NotFound,
    Unauthorized,
    TooManyRequests,
}

impl ResponseError {
    pub fn from_error(err: String) -> Self {
        Self::InternalServerError(err)
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        match self {
            ResponseError::Bad(message) => (
                StatusCode::BAD_REQUEST, with_message(Some(json_error("bad_request")), &message)
            ),
            ResponseError::InternalServerError(err) => {
                error!("{err}");
                (StatusCode::INTERNAL_SERVER_ERROR, json_error("internal_server_error"))
            },
            ResponseError::NotFound => (StatusCode::NOT_FOUND, with_message(None, "not_found")),
            ResponseError::Unauthorized => (StatusCode::UNAUTHORIZED, json_error("unauthorized")),
            ResponseError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, json_error("too_many_requests")),
        }.into_response()
    }
}

fn json_error(error: &str) -> Json<Value> {
    Json(json!({"error": error}))
}

fn with_message(json: Option<Json<Value>>, message: &str) -> Json<Value> {
    let mut json = match json {
        None => json!({}),
        Some(Json(json)) => json
    };
    json["message"] = json!(message);
    Json(json)
}
