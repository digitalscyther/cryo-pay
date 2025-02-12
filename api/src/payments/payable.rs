use std::sync::Arc;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
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
    pub target: SubscriptionTarget,
    pub until: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTarget {
    HighPriorityBlockchainChecking,     // high_priority_blockchain_checking
    PrivateInvoices,                    // private_invoices
    UnlimitedInvoices                   // unlimited_invoices
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Donation {
    pub donor: Option<String>,
    pub target: Option<String>,
    pub amount: BigDecimal,
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
        let as_value = serde_json::to_value(value).unwrap_or_default();
        serde_json::from_value(as_value).unwrap_or_default()
    }
}

impl TryInto<SubscriptionTarget> for String {
    type Error = ();

    fn try_into(self) -> Result<SubscriptionTarget, Self::Error> {
        let as_value = serde_json::to_value(self).map_err(|_| ())?;
        serde_json::from_value(as_value).map_err(|_| ())
    }
}

impl SubscriptionTarget {
    pub fn price_per_day(&self) -> BigDecimal {
        BigDecimal::from(match self {
            SubscriptionTarget::HighPriorityBlockchainChecking => 100,
            SubscriptionTarget::PrivateInvoices => 16,
            SubscriptionTarget::UnlimitedInvoices => 1
        }) / 100
    }

    pub fn iterator() -> Vec<Self> {
        vec![
            Self::HighPriorityBlockchainChecking,
            Self::UnlimitedInvoices,
            Self::PrivateInvoices,
        ]
    }
}

pub async fn apply(state: &Arc<AppState>, payment: &Payment) -> Result<(), String> {
    state.db.set_payment_paid(&payment.id).await?;

    let payable = serde_json::from_value::<Payable>(payment.data.clone())
        .map_err(|err| utils::make_err(Box::new(err), "parse payment"))?;

    match payable {
        Payable::Donation(_) => {},
        Payable::Subscription(subscription) => {
            let user_id = payment.user_id.ok_or_else(|| "subscription user not detected".to_string())?;
            let target: String = subscription.target.into();
            state.db.create_or_update_subscription(&user_id, &target, None, subscription.until).await?;
        }
    }

    Ok(())
}
