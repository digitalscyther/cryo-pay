use std::sync::Arc;
use axum::{Json, middleware, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use tracing::debug;
use crate::api::middleware::extract_user;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::{AppState, VerifyError};

const JWT_EXPIRE_DAYS: i64 = 7;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route(
            "/login",
            post(login)
                .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::login))
                .layer(middleware::from_fn_with_state(app_state.clone(), extract_user)),
        )
        .route("/logout", post(logout))
        .with_state(app_state)
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(crate) struct FirebaseTokenRequest {
    pub token: String,
}

fn build_jwt_cookie(value: String) -> Cookie<'static> {
    let mut cookie = Cookie::new("jwt", value);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = FirebaseTokenRequest,
    responses(
        (status = 200, description = "Authenticated, JWT cookie set"),
        (status = 400, description = "Invalid token"),
    ),
    tag = "auth"
)]
pub(crate) async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<FirebaseTokenRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    let token = state.gc.get_verified_token(&payload.token)
        .await
        .map_err(|err| match err {
            VerifyError::Verification(tve) => {
                debug!("Invalid token: {:?}", tve);
                ResponseError::Bad("Invalid token".to_string())
            }
            VerifyError::Unexpected(err) => ResponseError::from_error(err)
        })?;

    let user_id = token.critical_claims.sub;
    let email = token.all_claims
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let jwt = state.jwt.generate(user_id, email)
        .map_err(|err| ResponseError::from_error(format!("{err:?}")))?;

    let mut cookie = build_jwt_cookie(jwt);
    cookie.set_expires(OffsetDateTime::now_utc() + Duration::days(JWT_EXPIRE_DAYS));

    Ok((StatusCode::OK, jar.add(cookie)))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out, JWT cookie cleared"),
    ),
    tag = "auth"
)]
pub(crate) async fn logout(
    jar: CookieJar,
) -> Result<impl IntoResponse, StatusCode> {
    let mut cookie = build_jwt_cookie("".to_owned());
    cookie.set_max_age(Duration::seconds(0));

    Ok((StatusCode::OK, jar.remove(cookie)))
}
