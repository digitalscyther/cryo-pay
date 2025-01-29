use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
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
enum SubscriptionTarget {
    Blank
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Donation {
    donor: Option<String>,
    target: Option<String>,
    amount: BigDecimal,
}

impl Payable {
    fn create_subscription_blank() -> Self {
        Self::create_subscription(Subscription::blank())
    }

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
    fn blank() -> Self {
        Subscription::new(SubscriptionTarget::Blank, Utc::now().naive_utc())
    }

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
