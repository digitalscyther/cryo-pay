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


pub fn get_router(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/", get(list))
        .route("/", post(create))
        .route("/:invoice_id", get(read))
        .route("/:invoice_id", delete(destroy))
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<GetApiKeyResponse>>();

    Ok(Json(api_keys))
}

#[derive(Serialize)]
struct CreateResponse {
    api_key: String,
}

impl CreateResponse {
    fn new(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }
}

async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, StatusCode> {
    let api_key = utils::new_api_key(user.id);

    state.db
        .create_api_key(&user.id, &api_key.hashed_value())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateResponse::new(&api_key.value)))
}

async fn read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    if let Some(api_key) = state.db
        .get_api_key(&api_key_id, &user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let resp: GetApiKeyResponse = api_key.into();
        return Ok(Json(resp));
    }

    Err(StatusCode::NOT_FOUND)
}

async fn destroy(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.db.delete_api_key(&api_key_id, &user.id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}