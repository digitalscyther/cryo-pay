use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
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
        .route("/price", get(get_price))
        .route("/ping", get(ping_pong))
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    target: String,
    days: u64,
}

impl CreateSubscriptionRequest {
    fn validate(&self, max_days: u64) -> Result<(), ResponseError> {
        if self.days > max_days {
            return Err(ResponseError::Bad(format!("Max days: {}", max_days)))
        };

        Ok(())
    }

    fn subscription_target(&self) -> Result<SubscriptionTarget, String> {
        self.target.clone()
            .try_into()
            .map_err(|_| format!("Unknown target: {}", self.target))
    }

    fn get_price(&self) -> Result<BigDecimal, ResponseError> {
        self.subscription_target()
            .map(|target| calculate_price(&target, self.days))
            .map_err(ResponseError::Bad)
    }
}

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Json(request_data): Json<CreateSubscriptionRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    request_data.validate(45)?;
    let target= request_data.subscription_target()
        .map_err(ResponseError::from_error)?;
    let price = calculate_price(&target, request_data.days);
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

fn calculate_price(subscription_target: &SubscriptionTarget, days: u64) -> BigDecimal {
    subscription_target.price_per_day() * days
}

async fn get_price(
    Query(request_data): Query<CreateSubscriptionRequest>
) -> Result<impl IntoResponse, ResponseError> {
    request_data.validate(45)?;
    let price = request_data.get_price()?;
    Ok(Json(json!({"price": price})).into_response())
}
