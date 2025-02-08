use std::sync::Arc;
use axum::{Extension, middleware, Router};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use crate::api::buy::to_pay;
use crate::api::middleware::auth::AppUser;
use crate::api::middleware::extract_user;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::payments::ToPay;


pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(donate))
        .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::user_invoice))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .with_state(app_state.clone())
        .route("/ping", get(ping_pong))
}


#[derive(Deserialize)]
struct DonateRequest {
    amount: BigDecimal,
}

async fn donate(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<DonateRequest>,
    Extension(app_user): Extension<AppUser>,
) -> Result<impl IntoResponse, ResponseError> {
    let to_pay = ToPay::create_donation(payload.amount)
        .await
        .map_err(ResponseError::from_error)?;

    let payment_url = to_pay::create_payment_url(&to_pay, &state.db, app_user.user_id())
        .await
        .map_err(ResponseError::from_error)?;

    Ok(Redirect::to(&payment_url))
}