use std::sync::Arc;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Router;
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;
use crate::api::CALLBACK_PATH;
use crate::api::ping_pong::ping_pong;
use crate::api::response_error::ResponseError;
use crate::api::state::AppState;
use crate::db::billing::Payment;
use crate::payments::cryo_pay::{get_paid_payable, PaidPayableResult};
use crate::payments::payable::apply;
use crate::utils;

pub fn get_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(CALLBACK_PATH, get(callback))
        .route("/ping", get(ping_pong))
        .with_state(app_state)
}


#[derive(Deserialize)]
struct PaymentQuery {
    invoice_id: Uuid,
    status: String,
}

async fn callback(
    Query(payment_query): Query<PaymentQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ResponseError> {
    if payment_query.status != "SUCCESS" {
        return Err(ResponseError::Bad("wrong status".to_string()));
    }

    let payment = apply_paid_by_id(&state, &payment_query.invoice_id).await?;

    let redirect_url = utils::combine_paths(&[
        &utils::web_base_url().map_err(ResponseError::from_error)?,
        "/payment",
        &format!("/{}", payment.id),
    ]);

    Ok(Redirect::to(&redirect_url))
}

pub async fn apply_paid_by_id(state: &Arc<AppState>, id: &Uuid) -> Result<Payment, ResponseError> {
    match get_paid_payable(&state.db, id)
        .await
        .map_err(ResponseError::from_error)?
    {
        PaidPayableResult::NotPaid => Err(ResponseError::Bad("not paid".to_string())),
        PaidPayableResult::NotFound => Err(ResponseError::NotFound),
        PaidPayableResult::Payment(payment) => apply(&state, &payment)
            .await
            .map(|_| Ok(payment))
            .map_err(ResponseError::from_error)?
    }
}
