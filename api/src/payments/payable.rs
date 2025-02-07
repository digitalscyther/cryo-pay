use std::sync::Arc;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::api::state::AppState;
use crate::db::billing::Payment;
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payable {
    Subscription(Subscription),
    Donation(Donation)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    target: SubscriptionTarget,
    until: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTarget {
    InstantBlockchainChecking,
    PrivateInvoices,
    UnlimitedInvoices
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Donation {
    donor: Option<String>,
    target: Option<String>,
    amount: BigDecimal,
}

impl Payable {
    pub fn create_subscription(subscription: Subscription) -> Self {
        Self::Subscription(subscription)
    }

    pub fn create_anonymus_no_target_donation(amount: &BigDecimal) -> Self {
        Self::create_donation(Donation::anonymus_no_target(amount.clone()))
    }

    fn create_donation(donation: Donation) -> Self {
        Self::Donation(donation)
    }
}

impl Subscription {
    pub fn new(target: SubscriptionTarget, until: NaiveDateTime) -> Self {
        Self { target, until }
    }
}

impl Donation {
    fn anonymus_no_target(amount: BigDecimal) -> Self {
        Self::new(None, None, amount)
    }

    fn new(donor: Option<String>, target: Option<String>, amount: BigDecimal) -> Self {
        Self { donor, target, amount }
    }
}

impl From<SubscriptionTarget> for String {
    fn from(value: SubscriptionTarget) -> Self {
        serde_json::to_string(&value).unwrap_or_default()
    }
}

impl TryInto<SubscriptionTarget> for String {
    type Error = ();

    fn try_into(self) -> Result<SubscriptionTarget, Self::Error> {
        serde_json::from_str(&self).map_err(|_| ())
    }
}

impl SubscriptionTarget {
    pub fn price_per_day(&self) -> Result<BigDecimal, String> {
        match BigDecimal::from_f32(match self {
            SubscriptionTarget::InstantBlockchainChecking => 1.0,
            SubscriptionTarget::PrivateInvoices => 0.16,
            SubscriptionTarget::UnlimitedInvoices => 0.01
        }) {
            None => Err("Failed match price for sub".to_string()),
            Some(dec) => Ok(dec),
        }
    }
}

pub async fn apply(state: &Arc<AppState>, payment: Payment) -> Result<(), String> {
    info!("Payment #{} type={} got!", payment.id, payment.data);

    let payable = serde_json::from_value::<Payable>(payment.data)
        .map_err(|err| utils::make_err(Box::new(err), "parse payment"))?;

    match payable {
        Payable::Donation(_) => {},     // TODO add donation wall
        Payable::Subscription(subscription) => {
            let user_id = payment.user_id.ok_or_else(|| "subscription user not detected".to_string())?;
            let target: String = subscription.target.into();
            state.db.create_or_update_subscription(&user_id, &target, None, subscription.until).await?;
        }
    }

    Ok(())
}
