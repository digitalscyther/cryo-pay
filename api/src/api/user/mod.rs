use std::sync::Arc;
use axum::extract::State;
use axum::{Extension, Json, middleware, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, patch};
use serde::{Deserialize, Serialize};
use crate::api::db::User;
use crate::api::middleware::only_auth;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(read))
        .route("/", patch(update))
        .layer(middleware::from_fn_with_state(app_state.clone(), only_auth))
        .with_state(app_state)
}

#[derive(Deserialize)]
pub struct UserRequest {
    pub email_notification: Option<bool>,
    pub telegram_notification: Option<bool>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub with_chat_id: bool,
    pub email_notification: bool,
    pub telegram_notification: bool,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        UserResponse {
            with_chat_id: value.telegram_chat_id.is_some(),
            email_notification: value.email_notification,
            telegram_notification: value.telegram_notification
        }
    }
}

async fn read(Extension(user): Extension<User>) -> Result<impl IntoResponse, StatusCode> {
    let response: UserResponse = user.into();

    Ok(Json(response))
}

async fn update(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(payload): Json<UserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let response: UserResponse = state.db
        .update_user(&user.id, payload.email_notification, payload.telegram_notification)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into();

    Ok(Json(response))
}
