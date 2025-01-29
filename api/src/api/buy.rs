use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::Router;
use axum::routing::get;
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;
use crate::api::ping_pong::ping_pong;
use crate::api::state::AppState;
use crate::db::billing::Payment;
use crate::payments::{ToPay, ToPayId};
use crate::payments::payable::Payable;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/donate", get(donate))
        .with_state(app_state)
        .route("/ping", get(ping_pong))
}

#[derive(Deserialize)]
struct DonateRequest {
    amount: BigDecimal
}

struct ToPayAdapterPayment {
    invoice_id: Uuid,
    data: Value,
}

impl TryFrom<ToPay> for ToPayAdapterPayment {
    type Error = ();

    fn try_from(value: ToPay) -> Result<ToPayAdapterPayment, Self::Error> {
        let invoice_id = match value.id {
            ToPayId::CryoPay(id) => id
        };
        let data = serde_json::to_value(value.payable)
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
    Query(payload): Query<DonateRequest>
) -> Result<impl IntoResponse, StatusCode> {
    let to_pay = ToPay::create_donation(payload.amount)
        .await
        .map_err(|err| {
            warn!("Failed create ToPay: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let payment_url = to_pay.payment_url()
        .map_err(|err| {
            warn!("Failed get ToPay.payment_url: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let payment_adapter: ToPayAdapterPayment = to_pay.try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.db
        .create_payment(&payment_adapter.invoice_id, &payment_adapter.data)
        .await
        .map_err(|err| {
            warn!("Failed create Payment: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Redirect::to(&payment_url))
}