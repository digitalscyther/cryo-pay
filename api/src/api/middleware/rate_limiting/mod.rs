pub mod middleware;

use chrono::Utc;
use crate::api::middleware::auth::AppUser;
use crate::api::state::Redis;
use crate::error::AppError;

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

    pub async fn is_ok(&self, redis: &Redis, app_user: &AppUser) -> Result<bool, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_period_in_seconds_day() {
        assert_eq!(Period::Day.in_seconds(), 86400);
    }

    #[test]
    fn test_period_in_seconds_minute() {
        assert_eq!(Period::Minute.in_seconds(), 60);
    }

    #[test]
    fn test_create_10_times_per_day() {
        let rl = RateLimit::create_10_times_per_day(Target::ProductInvoice);
        assert!(matches!(rl.limit, Limit::Limited(10)));
        assert!(matches!(rl.period, Period::Day));
    }

    #[test]
    fn test_create_5_times_per_minute() {
        let rl = RateLimit::create_5_times_per_minute(Target::Login);
        assert!(matches!(rl.limit, Limit::Limited(5)));
        assert!(matches!(rl.period, Period::Minute));
    }
}
