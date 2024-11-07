use std::sync::Arc;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::cookie::Cookie;
use crate::api::state::AppState;

#[derive(Clone)]
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
