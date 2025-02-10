use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::api::buy::to_pay;
use crate::api::middleware::auth::AppUser;
use crate::api::middleware::extract_user;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::payments::ToPay;
use crate::utils;


pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(donate_create))
        .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::user_invoice))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .route("/", get(donate_list))
        .with_state(app_state.clone())
        .route("/ping", get(ping_pong))
}


#[derive(Deserialize)]
struct DonateRequest {
    amount: BigDecimal,
}

async fn donate_list(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, ResponseError> {
    let values = state.db.list_payment("donation")
        .await
        .map_err(ResponseError::from_error)? // Convert error from DB call to ResponseError
        .into_iter()
        .map(|p| {
            serde_json::to_value(p)
                .map_err(|err| utils::make_err(Box::new(err), "serializer payment"))
                .map(|mut value| {
                    value.as_object_mut().map(|obj| obj.remove("user_id"));
                    value
                })
        })
        .collect::<Result<Vec<Value>, _>>()
        .map_err(ResponseError::from_error)?;

    Ok(Json(values).into_response())
}


async fn donate_create(
    State(state): State<Arc<AppState>>,
    Extension(app_user): Extension<AppUser>,
    Json(payload): Json<DonateRequest>,
) -> Result<impl IntoResponse, ResponseError> {
    let to_pay = ToPay::create_donation(payload.amount)
        .await
        .map_err(ResponseError::from_error)?;

    let payment_url = to_pay::create_payment_url(&to_pay, &state.db, app_user.user_id())
        .await
        .map_err(ResponseError::from_error)?;

    Ok(Json(json!({"payment_url": payment_url})))
}