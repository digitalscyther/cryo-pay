pub mod middleware;

use chrono::Utc;
use crate::api::middleware::auth::AppUser;
use crate::api::state::Redis;

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
                .incr(
                    &format!("{}:{}", app_user.redis_key(), self.suffix()),
                    self.period.in_seconds()
                )
                .await
                .map(|done| times as u64 >= done)
        }
    }

    pub fn create_10_times_per_day(target: Target) -> Self {
        RateLimit {
            target,
            period: Period::Day,
            limit: Limit::Limited(10),
        }
    }

    pub fn create_5_times_per_minute(target: Target) -> Self {
        RateLimit {
            target,
            period: Period::Minute,
            limit: Limit::Limited(5),
        }
    }
}

#[derive(Debug)]
pub enum Target {
    ProductInvoice,
    UserInvoice,
    Login,
    CreateUserWebhook
}

pub enum Period {
    Day,
    Minute,
}

impl Period {
    fn suffix(&self) -> String {
        match self {
            Period::Day => format!("{}", Utc::now().format("%Y-%m-%d")),
            Period::Minute => format!("{}", Utc::now().format("%Y-%m-%dT%H:%M")),
        }
    }

    fn in_seconds(&self) -> u64 {
        match self {
            Period::Day => 24 * 3600,
            Period::Minute => 60
        }
    }
}
