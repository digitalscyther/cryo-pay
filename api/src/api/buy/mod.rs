mod subscription;

use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, middleware, Router};
use axum::routing::{get, post};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;
use crate::api::middleware::extract_user;
use crate::api::middleware::auth::AppUser;
use crate::api::middleware::rate_limiting::middleware::RateLimitType;
use crate::api::ping_pong::ping_pong;
use crate::api::state::{AppState, DB};
use crate::db::billing::Payment;
use crate::payments::{ToPay, ToPayId};
use crate::payments::cryo_pay::{get_paid_payable, PaidPayableResult};
use crate::payments::payable::{apply, Payable};

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/donate", get(donate)
            .layer(middleware::from_fn_with_state(app_state.clone(), RateLimitType::user_invoice)))
        .layer(middleware::from_fn_with_state(app_state.clone(), extract_user))
        .route("/recheck/:payment_id", post(recheck))
        .with_state(app_state.clone())
        .route("/ping", get(ping_pong))
        .nest("/subscription", subscription::get_router(app_state.clone()))
}

#[derive(Deserialize)]
struct DonateRequest {
    amount: BigDecimal,
}

struct ToPayAdapterPayment {
    invoice_id: Uuid,
    data: Value,
}

impl TryFrom<&ToPay> for ToPayAdapterPayment {
    type Error = ();

    fn try_from(value: &ToPay) -> Result<ToPayAdapterPayment, Self::Error> {
        let invoice_id = match value.id {
            ToPayId::CryoPay(id) => id
        };
        let data = serde_json::to_value(&value.payable)
            .map_err(|err| {
                warn!("Failed parse value to Payable: {err}");
                ()
            })?;

        Ok(ToPayAdapterPayment { invoice_id, data })
    }
}

impl TryFrom<Payment> for ToPay {
    type Error = ();

    fn try_from(value: Payment) -> Result<ToPay, Self::Error> {
        let payable: Payable = serde_json::from_value(value.data)
            .map_err(|err| {
                warn!("Failed parse value to Payable: {err}");
                ()
            })?;

        Ok(ToPay { id: ToPayId::CryoPay(value.id), payable })
    }
}

async fn donate(
    State(state): State<Arc<AppState>>,
    Query(payload): Query<DonateRequest>,
    Extension(app_user): Extension<AppUser>,
) -> Result<impl IntoResponse, StatusCode> {
    let to_pay = ToPay::create_donation(payload.amount)
        .await
        .map_err(|err| {
            warn!("Failed create ToPay: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let payment_url = create_payment_url(&to_pay, &state.db, app_user.user_id())
        .await
        .map_err(|err| {
            warn!("Failed create_payment_url: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Redirect::to(&payment_url))
}

async fn create_payment_url(to_pay: &ToPay, db: &DB, user_id: Option<Uuid>) -> Result<String, String> {
    let payment_url = to_pay.payment_url()?;

    let payment_adapter: ToPayAdapterPayment = to_pay
        .try_into()
        .map_err(|_| "Failed to_pay to Adapter".to_string())?;

    db.create_payment(&payment_adapter.invoice_id, user_id, &payment_adapter.data).await?;

    Ok(payment_url)
}

async fn recheck(
    State(state): State<Arc<AppState>>,
    Path(payment_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(match get_paid_payable(&state.db, &payment_id).await {
        Err(err) => {
            warn!("{err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Ok(PaidPayableResult::NotPaid) => StatusCode::BAD_REQUEST,
        Ok(PaidPayableResult::NotFound) => StatusCode::NOT_FOUND,
        Ok(PaidPayableResult::Payment(payment)) => match apply(&state, payment).await {
            Err(err) => {
                warn!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Ok(_) => StatusCode::OK
        },
    })
}