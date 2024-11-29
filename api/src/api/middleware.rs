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


const API_RPD: u64 = 1000;
const WEB_RPD: u64 = 1000;
const ANONYMUS_RPD: u64 = 1000;


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

    fn create(u: Option<User>, ip: String) -> Self {
        match u {
            None => MaybeUser::Anonymus(ip),
            Some(u) => MaybeUser::AuthUser(AuthUser::Web(u))
        }
    }
}

async fn get_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Real-IP")
        .and_then(|real_ip| real_ip.to_str().ok())
        .map(|real_ip_str| real_ip_str.to_string())
}

async fn get_user_from_headers(headers: &HeaderMap, state: Arc<AppState>) -> Option<User> {
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

pub async fn extract_jwt(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let user = get_user_from_headers(req.headers(), state).await;
    let ip;
    if let Some(_ip) = get_ip_from_headers(req.headers()).await {
        ip = _ip;
    } else {
        ip = "-".to_string();
    };
    let to_insert: MaybeUser = MaybeUser::create(user, ip);

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