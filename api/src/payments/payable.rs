use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::db::billing::Payment;
use crate::payments::donation::Donation;
use crate::payments::subscription::Subscription;
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payable {
    Subscription(Subscription),
    Donation(Donation),
}

pub async fn apply(state: &Arc<AppState>, payment: &Payment) -> Result<(), String> {
    state.db.set_payment_paid(&payment.id).await?;

    let payable = serde_json::from_value::<Payable>(payment.data.clone())
        .map_err(|err| utils::make_err(Box::new(err), "parse payment"))?;

    match payable {
        Payable::Donation(_) => {}
        Payable::Subscription(subscription) => {
            let user_id = payment.user_id.ok_or_else(|| "subscription user not detected".to_string())?;
            let target: String = subscription.target.into();
            state.db.create_or_update_subscription(&user_id, &target, None, subscription.until).await?;
        }
    }

    Ok(())
}
