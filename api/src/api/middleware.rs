use std::sync::Arc;
use axum::Extension;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;
use crate::api::state::AppState;
use crate::db::User;
use crate::utils;


const API_RPD: u64 = 1000;
const WEB_RPD: u64 = 1000;
const ANONYMUS_RPD: u64 = 1000;

#[derive(Clone, Debug)]
pub enum AuthType {
    API,
    WEB,
}

impl AuthType {
    fn rpd(&self) -> u64{
        match self {
            AuthType::API => API_RPD,
            AuthType::WEB => WEB_RPD,
        }
    }

    fn is_web(&self) -> bool {
        match self {
            AuthType::WEB => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Auth {
    auth_type: AuthType,
    user: User,
}

impl Auth {
    fn new(auth_type: AuthType, user: User) -> Self {
        Self { auth_type, user }
    }

    fn redis_key(&self) -> String {
        format!("{:?}:{}", self.auth_type, self.user.id)
    }

    fn rpd(&self) -> u64 {
        self.auth_type.rpd()
    }
}


#[derive(Clone, Debug)]
pub struct AppUser {
    ip: String,
    auth: Option<Auth>,
}

impl AppUser {
    pub fn new(ip: String, auth: Option<Auth>) -> Self {
        Self { ip, auth }
    }

    pub fn user_id(&self) -> Option<Uuid> {
        if let Some(auth) = &self.auth {
            return Some(auth.user.id)
        };

        None
    }

    fn rate_limit_redis_key(&self) -> String {
        let redis_key_suffix = match &self.auth {
            Some(auth) => auth.redis_key(),
            None => format!("anonymus:{}", self.ip),
        };

        format!("rate_limit:{}", redis_key_suffix)
    }

    fn rpd(&self) -> u64 {
        match &self.auth {
            Some(auth) => auth.rpd(),
            None => ANONYMUS_RPD,
        }
    }
}

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
) -> Result<impl IntoResponse, StatusCode> {
    match app_user.auth {
        Some(auth) => only(auth.user, req, next).await,
       _ => Err(StatusCode::UNAUTHORIZED)
    }
}

async fn only(
    user: User,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

pub async fn only_web(
    Extension(app_user): Extension<AppUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match app_user.auth {
        Some(auth) if auth.auth_type.is_web() => only(auth.user, req, next).await,
       _ => Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn is_rpd_ok(state: Arc<AppState>, key: &str, rpd_allowed: u64) -> Result<bool, String> {
    let now = Utc::now();
    let key = format!("{}:{}", key, now.format("%Y-%m-%d"));

    state.redis.incr(&key, 24 * 3600)
        .await
        .map(|rpd_done| rpd_allowed >= rpd_done)
}

pub async fn rate_limit(
    State(state): State<Arc<AppState>>,
    Extension(app_user): Extension<AppUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match is_rpd_ok(state, &app_user.rate_limit_redis_key(), app_user.rpd()).await {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err(StatusCode::TOO_MANY_REQUESTS),
        Err(e) => {
            error!("{}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}