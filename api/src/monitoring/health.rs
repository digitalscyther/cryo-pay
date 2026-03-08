use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

const STALE_THRESHOLD_SECS: i64 = 300; // 5 minutes
pub const UNHEALTHY_ERROR_THRESHOLD: u32 = 10;

pub struct DaemonHealth {
    healthy: AtomicBool,
    last_success_ts: AtomicI64,
}

impl DaemonHealth {
    pub fn new() -> Self {
        Self {
            healthy: AtomicBool::new(true),
            last_success_ts: AtomicI64::new(0),
        }
    }

    pub fn record_success(&self) {
        self.healthy.store(true, Ordering::Relaxed);
        let now = chrono::Utc::now().timestamp();
        self.last_success_ts.store(now, Ordering::Relaxed);
    }

    pub fn record_failure(&self, consecutive_errors: u32) {
        if consecutive_errors >= UNHEALTHY_ERROR_THRESHOLD {
            self.healthy.store(false, Ordering::Relaxed);
        }
    }

    pub fn is_healthy(&self) -> bool {
        let healthy = self.healthy.load(Ordering::Relaxed);
        let last_ts = self.last_success_ts.load(Ordering::Relaxed);

        if last_ts == 0 {
            // Daemon hasn't completed first cycle yet — consider healthy (starting up)
            return healthy;
        }

        let now = chrono::Utc::now().timestamp();
        healthy && (now - last_ts) < STALE_THRESHOLD_SECS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_starts_healthy() {
        let health = DaemonHealth::new();
        assert!(health.is_healthy());
    }

    #[test]
    fn test_record_success_keeps_healthy() {
        let health = DaemonHealth::new();
        health.record_success();
        assert!(health.is_healthy());
    }

    #[test]
    fn test_failure_below_threshold_stays_healthy() {
        let health = DaemonHealth::new();
        health.record_success();
        health.record_failure(UNHEALTHY_ERROR_THRESHOLD - 1);
        assert!(health.is_healthy());
    }

    #[test]
    fn test_failure_at_threshold_marks_unhealthy() {
        let health = DaemonHealth::new();
        health.record_success();
        health.record_failure(UNHEALTHY_ERROR_THRESHOLD);
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_success_after_failure_restores_healthy() {
        let health = DaemonHealth::new();
        health.record_success();
        health.record_failure(UNHEALTHY_ERROR_THRESHOLD);
        assert!(!health.is_healthy());
        health.record_success();
        assert!(health.is_healthy());
    }
}
