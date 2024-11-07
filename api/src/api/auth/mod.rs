use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use time::Duration;
use tracing::{debug, warn};
use crate::api::middleware::{extract_jwt, log_jwt, User};
use crate::api::ping_pong::ping_pong;
use crate::api::state::{AppState, VerifyError};

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/login", post(login))
        .route("/logout", post(logout)
            .layer(middleware::from_fn(log_jwt))
            .layer(middleware::from_fn_with_state(app_state.clone(), extract_jwt)))
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
    cookie.set_same_site(SameSite::Strict);

    Ok((StatusCode::OK, jar.add(cookie)))
}

async fn logout(
    jar: CookieJar,
    Extension(user): Extension<Option<User>>,
) -> impl IntoResponse {
    if user.is_none() {
        return Err(StatusCode::OK);
    }

    let mut cookie = Cookie::new("jwt", "");
    cookie.set_max_age(Duration::seconds(0));
    Ok((StatusCode::OK, jar.remove(cookie)))
}
