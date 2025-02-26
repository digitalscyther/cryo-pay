use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::api::response_error::ResponseError;
use crate::db::{User, Webhook};

const WEBHOOKS_LIMIT: usize = 2;

pub fn get_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create))
        .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::user_webhook))
        .route("/", get(list))
        .route("/:webhook_id", delete(destroy))
        .with_state(app_state)
        .route("/ping", get(ping_pong))
}

#[derive(Serialize)]
struct GetWebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
}

impl From<Webhook> for GetWebhookResponse {
    fn from(value: Webhook) -> Self {
        Self {
            id: value.id,
            url: value.url,
            created_at: value.created_at,
        }
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, ResponseError> {
    let webhooks = state.db
        .list_webhooks(&user.id)
        .await
        .map_err(ResponseError::from_error)?
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<GetWebhookResponse>>();

    Ok(Json(webhooks))
}

#[derive(Deserialize)]
struct CreateWebhookRequest {
    url: String,
}

impl CreateWebhookRequest {
    async fn validate(&self) -> Result<(), String> {
        Url::parse(&self.url)
            .map_err(|_| "Invalid url")?;

        match reqwest::Client::new().head(&self.url).send().await {
            Ok(response) if response.status().is_success() => Ok(()),
            Ok(response) => Err(format!("URL returned status: {}", response.status())),
            Err(e) => Err(format!("Failed to reach URL: {}", e)),
        }
    }
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    payload
        .validate()
        .await
        .map_err(ResponseError::Bad)?;

    let webhooks_number = state.db
        .count_webhooks(&user.id)
        .await
        .map_err(ResponseError::from_error)?;

    if webhooks_number >= WEBHOOKS_LIMIT {
        return Err(ResponseError::Bad("too many webhooks".to_string()));
    }

    let instance: GetWebhookResponse = state.db
        .create_webhook(&payload.url, &user.id)
        .await
        .map_err(ResponseError::from_error)?
        .into();

    Ok(Json(instance))
}

async fn destroy(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(webhook_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(match state.db.delete_webhook(&webhook_id, &user.id)
        .await
        .map_err(ResponseError::from_error)? {
        true => StatusCode::NO_CONTENT,
        false => return Err(ResponseError::NotFound),
    })
}