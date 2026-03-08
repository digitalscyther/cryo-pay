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
use crate::api::response_error::ResponseError;


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

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct GetApiKeyResponse {
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

#[utoipa::path(
    get,
    path = "/user/api_key",
    responses(
        (status = 200, description = "List of API keys", body = Vec<GetApiKeyResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
    security(("jwt_cookie" = []))
)]
pub(crate) async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, ResponseError> {
    let api_keys = state.db
        .list_api_key(&user.id)
        .await
        .map_err(ResponseError::from)?
        .into_iter()
        .map(|i| i.into())
        .collect::<Vec<GetApiKeyResponse>>();

    Ok(Json(api_keys))
}

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct CreateApiKeyResponse {
    key: String,
    instance: GetApiKeyResponse,
}

impl CreateApiKeyResponse {
    fn new(api_key: &str, instance: GetApiKeyResponse) -> Self {
        Self { key: api_key.to_string(), instance }
    }
}

#[utoipa::path(
    post,
    path = "/user/api_key",
    responses(
        (status = 200, description = "Created API key (shown once)", body = CreateApiKeyResponse),
        (status = 400, description = "Too many api keys"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
    security(("jwt_cookie" = []))
)]
pub(crate) async fn create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
) -> Result<impl IntoResponse, ResponseError> {
    let api_keys_number = state.db
        .count_api_keys_by_user_id(&user.id)
        .await
        .map_err(ResponseError::from)?;

    if api_keys_number >= API_KEY_LIMIT {
        return Err(ResponseError::Bad("too many api keys".to_string()))
    }

    let api_key = utils::new_api_key(user.id);

    let instance: GetApiKeyResponse = state.db
        .create_api_key(&user.id, &api_key.hashed_value())
        .await
        .map_err(ResponseError::from)?
        .into();

    Ok(Json(CreateApiKeyResponse::new(&api_key.value, instance)))
}

#[utoipa::path(
    get,
    path = "/user/api_key/{id}",
    params(("id" = Uuid, Path, description = "API key ID")),
    responses(
        (status = 200, description = "API key info", body = GetApiKeyResponse),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
    security(("jwt_cookie" = []))
)]
pub(crate) async fn read(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    if let Some(api_key) = state.db
        .get_api_key(&api_key_id, &user.id)
        .await
        .map_err(ResponseError::from)? {
        let resp: GetApiKeyResponse = api_key.into();
        return Ok(Json(resp));
    }

    Err(ResponseError::NotFound)
}

#[utoipa::path(
    delete,
    path = "/user/api_key/{id}",
    params(("id" = Uuid, Path, description = "API key ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "user",
    security(("jwt_cookie" = []))
)]
pub(crate) async fn destroy(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<db::User>,
    Path(api_key_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(match state.db.delete_api_key(&api_key_id, &user.id)
        .await
        .map_err(ResponseError::from)? {
        true => StatusCode::NO_CONTENT,
        false => return Err(ResponseError::NotFound),
    })
}
