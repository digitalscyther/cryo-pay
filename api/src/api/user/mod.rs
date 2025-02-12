mod api_key;
mod callback_url;

use std::sync::Arc;
use axum::extract::State;
use axum::{Extension, Json, middleware, Router};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, patch};
use serde::{Deserialize, Serialize};
use crate::api::middleware::{extract_user, only_web};
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::api::USER_BASE_PATH;
use crate::db::{billing, User};
use crate::payments::payable::Subscription;

const ATTACH_TELEGRAM_PATH: &str = "/attach_telegram";

fn get_attach_telegram_full_path() -> String {
    format!("{}{}", USER_BASE_PATH, ATTACH_TELEGRAM_PATH)
}

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(get_user))
        .route("/", patch(update_user))
        .route(ATTACH_TELEGRAM_PATH, get(attach_telegram))
        .nest("/api_key", api_key::get_router(app_state.clone()))
        .nest("/callback_url", callback_url::get_router(app_state.clone()))
        .layer(middleware::from_fn_with_state(app_state.clone(), only_web))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .with_state(app_state)
}

#[derive(Deserialize)]
pub struct UserRequest {
    pub email_notification: Option<bool>,
    pub telegram_notification: Option<bool>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub attach_telegram_path: Option<String>,
    pub email_notification: bool,
    pub telegram_notification: bool,
    pub subscriptions: Vec<Subscription>,
}

impl UserResponse {
    fn with_subscriptions(self, subscriptions: Vec<Subscription>) -> Self {
        Self {
            subscriptions,
            ..self
        }
    }
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
            email_notification: value.email_notification,
            telegram_notification: value.telegram_notification,
            subscriptions: vec![],
        }
    }
}

impl TryFrom<billing::Subscription> for Subscription {
    type Error = ();

    fn try_from(value: billing::Subscription) -> Result<Self, Self::Error> {
        Ok(Self {
            target: value.target
                .try_into()
                .map_err(|_| ())?,
            until: value.until,
        })
    }
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, ResponseError> {
    let response: UserResponse = user.clone().into();

    let subscriptions: Result<Vec<_>, _> = state.db.list_user_subscriptions(&user.id)
        .await
        .map_err(ResponseError::from_error)?
        .into_iter()
        .map(|s| s.try_into().map_err(|_| "Failed to parse subscription"))
        .collect();

    let subscriptions = subscriptions.map_err(|e| ResponseError::from_error(e.to_string()))?;

    let response = response.with_subscriptions(subscriptions);

    Ok(Json(response))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(payload): Json<UserRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    let response: UserResponse = state.db
        .update_user(&user.id, payload.email_notification, payload.telegram_notification)
        // .update_user(&user.id, None, payload.telegram_notification)     // TODO notification_turned_off
        .await
        .map_err(ResponseError::from_error)?
        .into();

    Ok(Json(response))
}

async fn attach_telegram(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, ResponseError> {
    let telegram_bot_name = state.telegram_client.get_bot_name()
        .await
        .map_err(ResponseError::from_error)?;

    let telegram_redirect_url = format!(
        "https://t.me/{}?start={}",
        telegram_bot_name,
        user.id
    );

    Ok(Redirect::temporary(&telegram_redirect_url))
}
