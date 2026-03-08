use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub target: SubscriptionTarget,
    pub until: NaiveDateTime,
}

impl Subscription {
    pub fn new(target: SubscriptionTarget, until: NaiveDateTime) -> Self {
        Self { target, until }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionTarget {
    HighPriorityBlockchainChecking,
    PrivateInvoices,
    UnlimitedInvoices,
}

impl SubscriptionTarget {
    pub fn price_per_day(&self) -> BigDecimal {
        BigDecimal::from(match self {
            SubscriptionTarget::HighPriorityBlockchainChecking => 100,
            SubscriptionTarget::PrivateInvoices => 16,
            SubscriptionTarget::UnlimitedInvoices => 1,
        }) / 100
    }

    pub fn iterator() -> Vec<Self> {
        vec![
            // Self::HighPriorityBlockchainChecking,
            Self::UnlimitedInvoices,
            Self::PrivateInvoices,
        ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case(SubscriptionTarget::HighPriorityBlockchainChecking, "1.00")]
    #[case(SubscriptionTarget::PrivateInvoices, "0.16")]
    #[case(SubscriptionTarget::UnlimitedInvoices, "0.01")]
    fn test_price_per_day(#[case] target: SubscriptionTarget, #[case] expected: &str) {
        let expected = BigDecimal::from_str(expected).unwrap();
        assert_eq!(target.price_per_day(), expected);
    }

    #[test]
    fn test_iterator_contains_expected_targets() {
        let targets = SubscriptionTarget::iterator();
        assert!(targets.contains(&SubscriptionTarget::UnlimitedInvoices));
        assert!(targets.contains(&SubscriptionTarget::PrivateInvoices));
    }

    #[rstest]
    #[case(SubscriptionTarget::PrivateInvoices, "private_invoices")]
    #[case(SubscriptionTarget::UnlimitedInvoices, "unlimited_invoices")]
    #[case(SubscriptionTarget::HighPriorityBlockchainChecking, "high_priority_blockchain_checking")]
    fn test_subscription_target_string_roundtrip(#[case] target: SubscriptionTarget, #[case] expected_str: &str) {
        let s: String = target.clone().into();
        assert_eq!(s, expected_str);
        let back: SubscriptionTarget = s.try_into().unwrap();
        assert_eq!(back, target);
    }

    #[test]
    fn test_invalid_target_string() {
        let result: Result<SubscriptionTarget, ()> = "nonexistent".to_string().try_into();
        assert!(result.is_err());
    }
}
