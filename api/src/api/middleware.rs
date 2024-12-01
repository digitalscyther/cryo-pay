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


pub enum AuthType {
    Api,
    Web,
}

#[derive(Clone, Debug)]
pub enum AuthUser {
    Api(User),
    Web(User),
}

impl AuthUser {
    fn user(&self) -> User {
        match self {
            AuthUser::Api(u) | AuthUser::Web(u) => u
        }.to_owned()
    }

    fn rate_limit_key(&self) -> String {
        let key = match self {
            AuthUser::Api(_) => "api",
            AuthUser::Web(_) => "web",
        };
        format!("rate_limit:{}:{}", key, self.user_id())
    }

    fn rpd_allowed(&self) -> u64 {
        match self {
            AuthUser::Api(_) => API_RPD,
            AuthUser::Web(_) => WEB_RPD,
        }
    }

    fn user_id(&self) -> Uuid {
        self.user().id
    }
}


#[derive(Clone, Debug)]
pub enum MaybeUser {
    AuthUser(AuthUser),
    Anonymus(String),
}

impl MaybeUser {
    pub fn user_id(&self) -> Option<Uuid> {
        match self {
            MaybeUser::AuthUser(auth_user) => Some(auth_user.user_id().to_owned()),
            MaybeUser::Anonymus(_) => None,
        }
    }

    fn rate_limit_key(&self) -> String {
        match self {
            MaybeUser::AuthUser(auth_user) => auth_user.rate_limit_key(),
            MaybeUser::Anonymus(ip) => format!("rate_limit:anonymus:{}", ip)
        }
    }

    fn rpd_allowed(&self) -> u64 {
        match self {
            MaybeUser::AuthUser(auth_user) => auth_user.rpd_allowed(),
            MaybeUser::Anonymus(_) => ANONYMUS_RPD,
        }
    }
}

async fn get_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Real-IP")
        .and_then(|real_ip| real_ip.to_str().ok())
        .map(|real_ip_str| real_ip_str.to_string())
}

struct Auth {
    type_: AuthType,
    user: User,
}

impl Auth {
    fn new(type_: AuthType, user: User) -> Self {
        Self { type_, user }
    }
}

impl From<(Option<Auth>, String)> for MaybeUser {
    fn from(value: (Option<Auth>, String)) -> Self {
        let (auth, ip) = value;
        match auth {
            None => MaybeUser::Anonymus(ip),
            Some(auth) => match auth.type_ {
                AuthType::Api => MaybeUser::AuthUser(AuthUser::Api(auth.user)),
                AuthType::Web => MaybeUser::AuthUser(AuthUser::Web(auth.user))
            }
        }
    }
}

async fn get_auth_from_headers(headers: &HeaderMap, state: Arc<AppState>) -> Option<Auth> {
    if let Some(user) = get_web_user_from_headers(headers, state.clone()).await {
        return Some(Auth::new(AuthType::Web, user));
    };

    if let Some(user) = get_api_user_from_headers(headers, state).await {
        return Some(Auth::new(AuthType::Api, user));
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


pub async fn extract_maybe_user(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let ip = get_ip_from_headers(req.headers()).await.unwrap_or("-".to_string());

    let auth = get_auth_from_headers(req.headers(), state).await;
    let to_insert: MaybeUser = (auth, ip).into();

    req.extensions_mut().insert(to_insert);
    next.run(req).await
}

pub async fn only_auth(
    Extension(maybe_user): Extension<MaybeUser>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match maybe_user {
        MaybeUser::AuthUser(auth_user) => {
            req.extensions_mut().insert(auth_user.user());
            Ok(next.run(req).await)
        }
        MaybeUser::Anonymus(_) => Err(StatusCode::UNAUTHORIZED)
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
    Extension(maybe_user): Extension<MaybeUser>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    match is_rpd_ok(state, &maybe_user.rate_limit_key(), maybe_user.rpd_allowed()).await {
        Ok(true) => Ok(next.run(req).await),
        Ok(false) => Err(StatusCode::TOO_MANY_REQUESTS),
        Err(e) => {
            error!("{}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}