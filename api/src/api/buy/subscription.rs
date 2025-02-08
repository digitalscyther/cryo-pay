use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use chrono::{Duration, Utc};
use serde::Deserialize;
use crate::api::buy::to_pay::create_payment_url;
use crate::api::middleware::{extract_user, only_auth};
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::User;
use crate::payments::payable::{Payable, Subscription, SubscriptionTarget};
use crate::payments::ToPay;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_subscription))
        .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::user_invoice))
        .layer(middleware::from_fn_with_state(app_state.clone(), only_auth))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .with_state(app_state)
        .route("/ping", get(ping_pong))
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    target: String,
    days: u64,
}

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(request_data): Json<CreateSubscriptionRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    let target: SubscriptionTarget = request_data.target
        .try_into()
        .map_err(|_| ResponseError::Bad("Unknown target"))?;
    let price = target.price_per_day()
        .map_err(ResponseError::from_error)? * request_data.days;
    let until = (Utc::now() + Duration::days(request_data.days as i64)).naive_utc();
    let subscription = Subscription::new(target.clone(), until);
    let payable = Payable::create_subscription(subscription);

    let to_pay = ToPay::create(
        price,
        Some(format!("Subscription #{:?} for {} days (until={})", target, request_data.days, until)),
        payable
    )
        .await
        .map_err(ResponseError::from_error)?;

    let payment_url = create_payment_url(&to_pay, &state.db, Some(user.id))
        .await
        .map_err(ResponseError::from_error)?;

    Ok(Redirect::to(&payment_url))
}


