use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::db;
use crate::api::response_error::ResponseError;


const CALLBACK_URLS_LIMIT: usize = 5;


pub fn get_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(list))
        .route("/", post(create))
        .route("/:callback_url_id", delete(destroy))
        .with_state(app_state)
}

#[derive(Serialize)]
struct GetCallbackUrlResponse {
    pub id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
}

impl From<db::CallbackUrl> for GetCallbackUrlResponse {
    fn from(value: db::CallbackUrl) -> Self {
        Self {
            id: value.id,
            url: value.url,
            created_at: value.created_at,
        }
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, ResponseError> {
    let callback_urls = state.db
        .list_callback_urls(&user.id)
        .await
        .map_err(ResponseError::from_error)?
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<GetCallbackUrlResponse>>();

    Ok(Json(callback_urls))
}

#[derive(Deserialize)]
struct CreateCallbackUrlRequest {
    url: String,
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Json(payload): Json<CreateCallbackUrlRequest>
) -> Result<impl IntoResponse, ResponseError> {
    let callback_urls_number = state.db
        .count_callback_urls(&user.id)
        .await
        .map_err(ResponseError::from_error)?;

    if callback_urls_number >= CALLBACK_URLS_LIMIT {
        return Err(ResponseError::Bad("too many callback urls".to_string()))
    }

    let instance: GetCallbackUrlResponse = state.db
        .create_callback_url(&payload.url, &user.id)
        .await
        .map_err(ResponseError::from_error)?
        .into();

    Ok(Json(instance))
}

async fn destroy(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(callback_url_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(match state.db.delete_callback_url(&callback_url_id, &user.id)
        .await
        .map_err(ResponseError::from_error)? {
        true => StatusCode::NO_CONTENT,
        false => return Err(ResponseError::NotFound),
    })
}
