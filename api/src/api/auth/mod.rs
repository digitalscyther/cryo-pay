use std::sync::Arc;
use axum::{Extension, Json, middleware::{self, Next}, Router};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use jsonwebtoken::{encode, Header, EncodingKey, DecodingKey, decode, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use time::Duration;
use tracing::{debug, info, warn};
use crate::api::ping_pong::ping_pong;
use crate::api::state::{AppState, VerifyError};
use crate::utils;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ping", get(ping_pong))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_jwt))
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

    let jwt = generate_jwt(token.critical_claims.sub, &state.jwt_secret)
        .map_err(|err| {
            warn!("Failed generate jwt: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut cookie = Cookie::new("jwt", jwt);
    cookie.set_same_site(SameSite::Strict);

    Ok((StatusCode::OK, jar.add(cookie)))
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone, Debug)]
struct User {
    id: String
}

fn generate_jwt(user_id: String, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

async fn logout(
    jar: CookieJar,
    Extension(user): Extension<Option<User>>
) -> impl IntoResponse {
    info!("user: {:?}", user);

    let mut cookie = Cookie::new("jwt", "");
    cookie.set_max_age(Duration::seconds(0));

    (StatusCode::OK, jar.remove(cookie))
}


impl Claims {
    fn from_jwt(token: &str, secret: &str) -> Result<Self, String> {
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let token_data = decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
            .map_err(|err| utils::make_err(Box::new(err), "decode jwt"))?;
        Ok(token_data.claims)
    }
}

async fn extract_jwt(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let user = req.headers()
        .get(axum::http::header::COOKIE)
        .and_then(|cookies| cookies.to_str().ok())
        .and_then(|cookie_str| Cookie::parse(cookie_str).ok())
        .and_then(|cookie| Claims::from_jwt(&cookie.value(), &state.jwt_secret).ok())
        .map(|claims| User { id: claims.sub });

    req.extensions_mut().insert(user);
    next.run(req).await
}
