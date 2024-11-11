use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use tracing::{debug, warn};
use crate::api::ping_pong::ping_pong;
use crate::api::state::{AppState, VerifyError};

const JWT_EXPIRE_DAYS: i64 = 7;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .with_state(app_state)
}

#[derive(Deserialize)]
struct FirebaseTokenRequest {
    token: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<FirebaseTokenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let token = state.gc.get_verified_token(&payload.token)
        .await
        .map_err(|err| match err {
            VerifyError::Verification(tve) => {
                debug!("Invalid token: {:?}", tve);
                StatusCode::BAD_REQUEST
            }
            VerifyError::Unexpected(err) => {
                warn!("Failed check token: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let jwt = state.jwt.generate(token.critical_claims.sub)
        .map_err(|err| {
            warn!("Failed generate jwt: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut cookie = Cookie::new("jwt", jwt);
    cookie.set_expires(OffsetDateTime::now_utc() + Duration::days(JWT_EXPIRE_DAYS));
    cookie.set_path("/");
    cookie.set_same_site(SameSite::None);   // TODO check is it good for prod

    Ok((StatusCode::OK, jar.add(cookie)))
}

async fn logout(
    jar: CookieJar,
) -> Result<impl IntoResponse, StatusCode> {

    let mut cookie = Cookie::new("jwt", "");
    cookie.set_max_age(Duration::seconds(0));
    cookie.set_path("/");
    cookie.set_same_site(SameSite::None);

    Ok((StatusCode::OK, jar.remove(cookie)))
}
