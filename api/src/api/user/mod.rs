use std::sync::Arc;
use axum::extract::State;
use axum::{Extension, Json, middleware, Router};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, patch};
use serde::{Deserialize, Serialize};
use crate::api::middleware::only_auth;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::api::USER_BASE_PATH;
use crate::db::User;

const ATTACH_TELEGRAM_PATH: &str = "/attach_telegram";

fn get_attach_telegram_full_path() -> String {
    format!("{}{}", USER_BASE_PATH, ATTACH_TELEGRAM_PATH)
}

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(read))
        .route("/", patch(update))
        .route(ATTACH_TELEGRAM_PATH, get(attach_telegram))
        .layer(middleware::from_fn_with_state(app_state.clone(), only_auth))
        .with_state(app_state)
}

#[derive(Deserialize)]
pub struct UserRequest {
    // pub email_notification: Option<bool>,   // TODO 123 when to turn on?
    pub telegram_notification: Option<bool>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub attach_telegram_path: Option<String>,
    // pub email_notification: bool,    // TODO 123
    pub telegram_notification: bool,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        let need_telegram = value.telegram_notification && value.telegram_chat_id.is_none();
        let attach_telegram_path = match need_telegram {
            true => Some(get_attach_telegram_full_path()),
            false => None
        };

        UserResponse {
            attach_telegram_path,
            // email_notification: value.email_notification,    // TODO 123
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
        // .update_user(&user.id, payload.email_notification, payload.telegram_notification)    // TODO 123
        .update_user(&user.id, None, payload.telegram_notification)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into();

    Ok(Json(response))
}

async fn attach_telegram(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>
) -> Result<impl IntoResponse, StatusCode> {
    let telegram_bot_name = state.telegram_client.get_bot_name()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let telegram_redirect_url = format!(
        "https://t.me/{}?start={}",
        telegram_bot_name,
        user.id
    );

    Ok(Redirect::temporary(&telegram_redirect_url))
}
