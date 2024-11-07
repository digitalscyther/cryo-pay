use std::sync::Arc;
use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use tracing::info;
use crate::api::state::AppState;

#[derive(Clone, Debug)]
pub struct User {
    id: String
}

pub async fn extract_jwt(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let user = req.headers()
        .get(axum::http::header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok())
        .and_then(|cookie_str| Cookie::parse(cookie_str).ok())
        .and_then(|cookie| state.jwt.claims_from_jwt(&cookie.value()).ok())
        .map(|claims| User { id: claims.sub });

    req.extensions_mut().insert(user);
    next.run(req).await
}

pub async fn log_jwt(
    Extension(user): Extension<Option<User>>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    info!("user_id: {:?}", user.and_then(|u| Some(u.id)));
    next.run(req).await
}