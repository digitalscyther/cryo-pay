pub mod auth;
pub mod rate_limiting;

use std::sync::Arc;
use axum::Extension;
use axum::extract::{Path, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use tracing::error;
use uuid::Uuid;
use auth::{AppUser, Auth, AuthType};
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::User;
use crate::utils;

async fn get_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Real-IP")
        .and_then(|real_ip| real_ip.to_str().ok())
        .map(|real_ip_str| real_ip_str.to_string())
}

async fn get_auth_from_headers(headers: &HeaderMap, state: Arc<AppState>) -> Option<Auth> {
    if let Some(user) = get_web_user_from_headers(headers, state.clone()).await {
        return Some(Auth::new(AuthType::WEB, user));
    };

    if let Some(user) = get_api_user_from_headers(headers, state).await {
        return Some(Auth::new(AuthType::API, user));
    };

    None
}

async fn get_web_user_from_headers(headers: &HeaderMap, state: Arc<AppState>) -> Option<User> {
    let claims = headers
        .get(axum::http::header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find_map(|s| {
                    let cookie = Cookie::parse(s.trim()).ok()?;
                    if cookie.name() == "jwt" {
                        Some(cookie)
                    } else {
                        None
                    }
                })
        })
        .and_then(|cookie| state.jwt.claims_from_jwt(&cookie.value()).ok());

    match claims {
        None => None,
        Some(claims) => match state.db.get_or_create_user(&claims.sub, claims.email).await {
            Ok(user) => Some(user),
            Err(err) => {
                error!(err);
                None
            }
        }
    }
}

async fn get_api_user_from_headers(headers: &HeaderMap, state: Arc<AppState>) -> Option<User> {
    let api_key = headers
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .map(|auth_str| auth_str.trim_start_matches("Bearer ").trim());

    if let Some(api_key) = api_key {
        if let Ok(Some(api_key)) = state.db
            .get_api_key_by_api_key(&utils::ApiKey::hash_value(api_key))
            .await
            .map_err(|e| {
                error!(e);
                None::<User>
            }) {
            if let Err(e) = state.db.update_api_key_last_used(&api_key.id).await {
                error!(e);
            };
            return state.db
                .get_user_by_id(&api_key.user_id)
                .await
                .map_err(|e| {
                    error!(e);
                    None::<User>
                })
                .ok();
        }
    };

    None
}


pub async fn extract_user(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let ip = get_ip_from_headers(req.headers()).await.unwrap_or("-".to_string());

    let auth = get_auth_from_headers(req.headers(), state).await;
    let app_user = AppUser::new(ip, auth);

    req.extensions_mut().insert(app_user);
    next.run(req).await
}

pub async fn only_auth(
    Extension(app_user): Extension<AppUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    match app_user.auth {
        Some(auth) => only(auth.user, req, next).await,
        _ => Err(ResponseError::Unauthorized)
    }
}

async fn only(
    user: User,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn only_web(
    Extension(app_user): Extension<AppUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    match app_user.auth {
        Some(auth) if auth.auth_type.is_web() => only(auth.user, req, next).await,
        _ => Err(ResponseError::Unauthorized)
    }
}

pub async fn only_bill_owner(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
    Extension(app_user): Extension<AppUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ResponseError> {
    if let Some(auth) = app_user.auth {
        if let Ok(true) = state.db.get_is_owner(&invoice_id, &auth.user.id).await {
            return Ok(next.run(req).await);
        }
    }

    Err(ResponseError::NotFound)
}
