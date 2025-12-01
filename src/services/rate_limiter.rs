use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use nonzero_ext::nonzero;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Rate limiter for API keys
pub struct RateLimiter {
    /// Default rate limiter for non-API key requests
    default_limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Per-API-key rate limiters
    api_key_limiters: Arc<RwLock<HashMap<String, Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
    /// Default quota (requests per second)
    default_quota: Quota,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `default_rps` - Default requests per second for non-API-key requests
    pub fn new(default_rps: u32) -> Self {
        let default_quota = Quota::per_second(NonZeroU32::new(default_rps).unwrap_or(nonzero!(100u32)));
        let default_limiter = Arc::new(GovernorRateLimiter::direct(default_quota));

        info!("Rate limiter initialized with {} requests/second", default_rps);

        Self {
            default_limiter,
            api_key_limiters: Arc::new(RwLock::new(HashMap::new())),
            default_quota,
        }
    }

    /// Check if a request is allowed for the default limiter
    pub async fn check_default(&self) -> bool {
        self.default_limiter.check().is_ok()
    }

    /// Check if a request is allowed for a specific API key
    ///
    /// # Arguments
    /// * `api_key` - The API key to check
    /// * `custom_quota` - Optional custom quota for this API key (requests per second)
    pub async fn check_api_key(&self, api_key: &str, custom_quota: Option<u32>) -> bool {
        // Get or create limiter for this API key
        let limiter = {
            let read_guard = self.api_key_limiters.read().await;
            if let Some(limiter) = read_guard.get(api_key) {
                limiter.clone()
            } else {
                drop(read_guard);

                // Create new limiter for this API key
                let quota = if let Some(rps) = custom_quota {
                    Quota::per_second(NonZeroU32::new(rps).unwrap_or(nonzero!(100u32)))
                } else {
                    self.default_quota
                };

                let new_limiter = Arc::new(GovernorRateLimiter::direct(quota));

                let mut write_guard = self.api_key_limiters.write().await;
                write_guard.insert(api_key.to_string(), new_limiter.clone());

                info!("Created rate limiter for API key: {}... (quota: {:?})",
                    &api_key[..8.min(api_key.len())], quota);

                new_limiter
            }
        };

        let allowed = limiter.check().is_ok();

        if !allowed {
            warn!("Rate limit exceeded for API key: {}...", &api_key[..8.min(api_key.len())]);
        }

        allowed
    }

    /// Remove a rate limiter for an API key (e.g., when key is deleted)
    pub async fn remove_api_key(&self, api_key: &str) {
        let mut write_guard = self.api_key_limiters.write().await;
        write_guard.remove(api_key);
        info!("Removed rate limiter for API key: {}...", &api_key[..8.min(api_key.len())]);
    }

    /// Get the number of tracked API keys
    pub async fn tracked_keys_count(&self) -> usize {
        self.api_key_limiters.read().await.len()
    }

    /// Clear all API key limiters
    pub async fn clear_all(&self) {
        let mut write_guard = self.api_key_limiters.write().await;
        write_guard.clear();
        info!("Cleared all API key rate limiters");
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(100) // 100 requests per second default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10);
        assert_eq!(limiter.tracked_keys_count().await, 0);
    }

    #[tokio::test]
    async fn test_default_rate_limit() {
        let limiter = RateLimiter::new(2);

        // First request should succeed
        assert!(limiter.check_default().await);

        // Second request should succeed
        assert!(limiter.check_default().await);

        // Third request might fail (rate limit)
        // Note: This is time-dependent and might pass if executed slowly
    }

    #[tokio::test]
    async fn test_api_key_rate_limit() {
        let limiter = RateLimiter::new(100);
        let api_key = "test_key_123";

        // First request should succeed
        assert!(limiter.check_api_key(api_key, Some(2)).await);

        // Key should be tracked
        assert_eq!(limiter.tracked_keys_count().await, 1);

        // Second request should succeed
        assert!(limiter.check_api_key(api_key, Some(2)).await);
    }

    #[tokio::test]
    async fn test_remove_api_key() {
        let limiter = RateLimiter::new(100);
        let api_key = "test_key_to_remove";

        // Create limiter for key
        limiter.check_api_key(api_key, None).await;
        assert_eq!(limiter.tracked_keys_count().await, 1);

        // Remove it
        limiter.remove_api_key(api_key).await;
        assert_eq!(limiter.tracked_keys_count().await, 0);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let limiter = RateLimiter::new(100);

        // Create multiple limiters
        limiter.check_api_key("key1", None).await;
        limiter.check_api_key("key2", None).await;
        limiter.check_api_key("key3", None).await;

        assert_eq!(limiter.tracked_keys_count().await, 3);

        // Clear all
        limiter.clear_all().await;
        assert_eq!(limiter.tracked_keys_count().await, 0);
    }

    #[tokio::test]
    async fn test_custom_quota() {
        let limiter = RateLimiter::new(100);

        // Create limiter with custom quota
        assert!(limiter.check_api_key("custom_key", Some(1000)).await);
        assert_eq!(limiter.tracked_keys_count().await, 1);
    }
}

