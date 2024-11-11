use std::sync::Arc;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use tracing::error;
use uuid::Uuid;
use crate::api::db::User;
use crate::api::state::AppState;

#[derive(Clone, Debug)]
pub enum MaybeUser {
    User(User),
    None,
}

impl MaybeUser {
    pub fn user_id(&self) -> Option<Uuid> {
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

pub async fn extract_jwt(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let claims = req.headers()
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

    let user = match claims {
        None => None,
        Some(claims) => match state.db.get_or_create_user(&claims.sub).await {
            Ok(user) => Some(user),
            Err(err) => {
                error!(err);
                None
            }
        }
    };
    let to_insert: MaybeUser = user.into();

    req.extensions_mut().insert(to_insert);
    next.run(req).await
}