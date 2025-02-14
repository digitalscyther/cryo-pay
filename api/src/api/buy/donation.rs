use std::sync::Arc;
use axum::{Extension, Json, middleware, Router};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use crate::api::buy::to_pay;
use crate::api::middleware::auth::AppUser;
use crate::api::middleware::extract_user;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::billing::Payment;
use crate::payments::payable::Payable;
use crate::payments::ToPay;


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

#[derive(Serialize)]
struct DonateItemResponse {
    id: Uuid,
    target: Option<String>,
    donor: Option<String>,
    amount: Option<BigDecimal>,
    paid_at: Option<NaiveDateTime>
}

impl TryFrom<Payment> for DonateItemResponse {
    type Error = ();

    fn try_from(value: Payment) -> Result<Self, Self::Error> {
        let payable: Payable = serde_json::from_value(value.data)
            .map_err(|_| ())?;

        let donation = match payable {
            Payable::Donation(donation) => Some(donation),
            _ => None
        };

        Ok(Self {
            id: value.id,
            target: donation.clone().map(|d| d.target).unwrap_or(None),
            donor: donation.clone().map(|d| d.donor).unwrap_or(None),
            amount: donation.clone().map(|d|d.amount).or(None),
            paid_at: value.paid_at
        })
    }
}

async fn donate_list(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DonateItemResponse>>, ResponseError> {
    let values = state.db.list_payment("donation", 100, 0)
        .await
        .map_err(ResponseError::from_error)?
        .into_iter()
        .map(|p| p.try_into())
        .collect::<Result<Vec<DonateItemResponse>, _>>()
        .map_err(|_| ResponseError::from_error("failed transform donations".to_string()))?;

    Ok(Json(values))
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