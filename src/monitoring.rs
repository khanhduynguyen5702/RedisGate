/// Basic monitoring and metrics tracking
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Global metrics tracking
#[derive(Clone)]
pub struct Metrics {
    /// Total requests received
    pub total_requests: Arc<AtomicU64>,

    /// Total successful responses
    pub total_success: Arc<AtomicU64>,

    /// Total errors
    pub total_errors: Arc<AtomicU64>,

    /// Redis commands executed
    pub redis_commands: Arc<AtomicU64>,

    /// Redis connection errors
    pub redis_connection_errors: Arc<AtomicU64>,

    /// API key validation failures
    pub auth_failures: Arc<AtomicU64>,

    /// Server start time
    pub start_time: DateTime<Utc>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_success: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            redis_commands: Arc::new(AtomicU64::new(0)),
            redis_connection_errors: Arc::new(AtomicU64::new(0)),
            auth_failures: Arc::new(AtomicU64::new(0)),
            start_time: Utc::now(),
        }
    }

    /// Increment request counter
    pub fn inc_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment success counter
    pub fn inc_success(&self) {
        self.total_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter
    pub fn inc_errors(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment Redis command counter
    pub fn inc_redis_commands(&self) {
        self.redis_commands.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment Redis connection error counter
    pub fn inc_redis_connection_errors(&self) {
        self.redis_connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment auth failure counter
    pub fn inc_auth_failures(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_success: self.total_success.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            redis_commands: self.redis_commands.load(Ordering::Relaxed),
            redis_connection_errors: self.redis_connection_errors.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            uptime_seconds: (Utc::now() - self.start_time).num_seconds(),
            timestamp: Utc::now(),
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_success: u64,
    pub total_errors: u64,
    pub redis_commands: u64,
    pub redis_connection_errors: u64,
    pub auth_failures: u64,
    pub uptime_seconds: i64,
    pub timestamp: DateTime<Utc>,
}

impl MetricsSnapshot {
    /// Calculate success rate (%)
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.total_success as f64 / self.total_requests as f64) * 100.0
    }

    /// Calculate error rate (%)
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.total_errors as f64 / self.total_requests as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        let metrics = Metrics::new();

        metrics.inc_requests();
        metrics.inc_requests();
        metrics.inc_success();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 2);
        assert_eq!(snapshot.total_success, 1);
    }

    #[test]
    fn test_success_rate() {
        let metrics = Metrics::new();

        metrics.inc_requests();
        metrics.inc_requests();
        metrics.inc_success();
        metrics.inc_success();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.success_rate(), 100.0);
    }

    #[test]
    fn test_error_rate() {
        let metrics = Metrics::new();

        metrics.inc_requests();
        metrics.inc_requests();
        metrics.inc_errors();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.error_rate(), 50.0);
    }
}

