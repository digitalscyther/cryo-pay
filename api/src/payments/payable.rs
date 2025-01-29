use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Payable {
    Subscription(Subscription),
    Donation(Donation)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    target: SubscriptionTarget,
    until: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
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
    fn create_subscription(subscription: Subscription) -> Self {
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
    fn new(target: SubscriptionTarget, until: NaiveDateTime) -> Self {
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
