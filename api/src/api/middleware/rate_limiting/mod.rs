pub mod middleware;

use chrono::Utc;
use crate::api::middleware::auth::AppUser;
use crate::api::state::Redis;

const REDIS_TIMEOUT: u64 = 24 * 3600;

pub enum Limit {
    Unlimited,
    Limited(u16),
}

pub struct RateLimit {
    pub target: Target,
    pub period: Period,
    pub limit: Limit,
}

impl RateLimit {
    fn suffix(&self) -> String {
        format!("rate-limit:{:?}:{}", self.target, self.period.suffix())
    }

    pub async fn is_ok(&self, redis: &Redis, app_user: &AppUser) -> Result<bool, String> {
        match self.limit {
            Limit::Unlimited => Ok(true),
            Limit::Limited(times) => redis
                .incr(&format!("{}:{}", app_user.redis_key(), self.suffix()), REDIS_TIMEOUT)
                .await
                .map(|done| times as u64 >= done)
        }
    }
}

#[derive(Debug)]
pub enum Target {
    ProductInvoice,
    UserInvoice,
}

pub enum Period {
    Day
}

impl Period {
    fn suffix(&self) -> String {
        match self {
            Period::Day => format!("{}", Utc::now().format("%Y-%m-%d"))
        }
    }
}
