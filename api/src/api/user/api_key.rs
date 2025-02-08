use std::sync::Arc;
use axum::{Extension, Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::{db, utils};


const API_KEY_LIMIT: usize = 5;


pub fn get_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(list))
        .route("/", post(create))
        .route("/:api_key_id", get(read))
        .route("/:api_key_id", delete(destroy))
        .with_state(app_state)
}

#[derive(Serialize)]
struct GetApiKeyResponse {
    pub id: Uuid,
    pub created: NaiveDateTime,
    pub last_used: Option<NaiveDateTime>,
}

impl From<db::ApiKey> for GetApiKeyResponse {
    fn from(value: db::ApiKey) -> Self {
        Self {
            id: value.id,
            created: value.created,
            last_used: value.last_used,
        }
    }
}

async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, StatusCode> {
    let api_keys = state.db
        .list_api_key(&user.id)
        .await
        .map_err(utils::log_and_error)?
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<GetApiKeyResponse>>();

    Ok(Json(api_keys))
}

#[derive(Serialize)]
struct CreateResponse {
    key: String,
    instance: GetApiKeyResponse,
}

impl CreateResponse {
    fn new(api_key: &str, instance: GetApiKeyResponse) -> Self {
        Self { key: api_key.to_string(), instance }
    }
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, StatusCode> {
    let api_keys_number = state.db
        .count_api_keys_by_user_id(&user.id)
        .await
        .map_err(utils::log_and_error)?;

    if api_keys_number >= API_KEY_LIMIT {
        return Err(StatusCode::CONFLICT)
    }

    let api_key = utils::new_api_key(user.id);

    let instance: GetApiKeyResponse = state.db
        .create_api_key(&user.id, &api_key.hashed_value())
        .await
        .map_err(utils::log_and_error)?
        .into();

    Ok(Json(CreateResponse::new(&api_key.value, instance)))
}

async fn read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(api_key) = state.db
        .get_api_key(&api_key_id, &user.id)
        .await
        .map_err(utils::log_and_error)? {
        let resp: GetApiKeyResponse = api_key.into();
        return Ok(Json(resp));
    }

    Err(StatusCode::NOT_FOUND)
}

async fn destroy(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(match state.db.delete_api_key(&api_key_id, &user.id)
        .await
        .map_err(utils::log_and_error)? {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::NOT_FOUND,
    })
}