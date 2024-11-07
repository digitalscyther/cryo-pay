use std::sync::Arc;
use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use tracing::info;
use crate::api::state::{AppState, Claims};

#[derive(Clone, Debug)]
pub enum MaybeUser {
    User(User),
    None
}

impl MaybeUser {
    pub fn is_none(&self) -> bool {
        matches!(self, MaybeUser::None)
    }

    pub fn user_id(&self) -> Option<String> {
        match self {
            MaybeUser::User(u) => Some(u.to_owned().id),
            MaybeUser::None => None,
        }
    }
}

impl From<Option<User>> for MaybeUser {
    fn from(u: Option<User>) -> Self {
        match u {
            None => MaybeUser::None,
            Some(u) => MaybeUser::User(u)
        }
    }
}

#[derive(Clone, Debug)]
pub struct User {
    id: String
}

impl From<Claims> for User {
    fn from(claims: Claims) -> Self {
        User { id: claims.sub }
    }
}

pub async fn extract_jwt(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let user: Option<User> = req.headers()
        .get(axum::http::header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok())
        .and_then(|cookie_str| Cookie::parse(cookie_str).ok())
        .and_then(|cookie| state.jwt.claims_from_jwt(&cookie.value()).ok())
        .map(|claims| claims.into());

    let to_insert: MaybeUser = user.into();
    req.extensions_mut().insert(to_insert);
    next.run(req).await
}

pub async fn log_jwt(
    Extension(user): Extension<MaybeUser>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    info!("user_id: {:?}", user.user_id());
    next.run(req).await
}