use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Donation {
    pub donor: Option<String>,
    pub target: Option<String>,
    pub amount: BigDecimal,
}

impl Donation {
    pub fn anonymous_no_target(amount: BigDecimal) -> Self {
        Self {
            donor: None,
            target: None,
            amount,
        }
    }
}
