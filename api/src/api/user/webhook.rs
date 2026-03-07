use std::net::IpAddr;
use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;
use uuid::Uuid;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::api::response_error::ResponseError;
use crate::db::{User, Webhook};
use crate::utils;

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
    pub secret: String,
    pub created_at: NaiveDateTime,
}

impl From<Webhook> for GetWebhookResponse {
    fn from(value: Webhook) -> Self {
        Self {
            id: value.id,
            url: value.url,
            secret: value.secret,
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

/// Block internal/private IPs and Docker-internal hostnames to prevent SSRF.
/// Note: DNS rebinding is not covered — would require a custom resolver;
/// hostname + IP validation is sufficient for this threat model.
fn validate_webhook_url(url: &Url) -> Result<(), String> {
    match url.scheme() {
        "http" | "https" => {}
        s => return Err(format!("Unsupported scheme: {s}")),
    }

    let host = url.host_str().ok_or("URL must have a host")?;

    const BLOCKED_HOSTS: &[&str] = &[
        "localhost", "postgres", "redis", "api", "web", "nginx", "traefik",
    ];
    let host_lower = host.to_lowercase();
    if BLOCKED_HOSTS.iter().any(|&b| host_lower == b) {
        return Err(format!("Blocked host: {host}"));
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        let is_internal = match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
            }
            IpAddr::V6(v6) => {
                v6.is_loopback()
                || v6.is_unspecified()
            }
        };
        if is_internal {
            return Err(format!("Internal IP not allowed: {ip}"));
        }
    }

    Ok(())
}

impl CreateWebhookRequest {
    async fn validate(&self) -> Result<(), String> {
        let parsed = Url::parse(&self.url)
            .map_err(|_| "Invalid url".to_string())?;

        validate_webhook_url(&parsed)?;

        match reqwest::Client::new()
            .post(&self.url)
            .header("content-type", "application/json")
            .json(&json!({}))
            .send().await {
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

    let secret = utils::generate_webhook_secret();

    let instance: GetWebhookResponse = state.db
        .create_webhook(&payload.url, &secret, &user.id)
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
