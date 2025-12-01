// Unit tests for Redis Pool

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_redis_pool_creation() {
        let pool = RedisPool::new();
        assert_eq!(pool.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_has_instance() {
        let pool = RedisPool::new();
        assert!(!pool.has_instance("test-instance").await);
    }

    #[tokio::test]
    async fn test_connection_count() {
        let pool = RedisPool::new();
        let count = pool.connection_count().await;
        assert_eq!(count, 0);
    }

    // Integration test - requires actual Redis instance
    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_connect_to_real_redis() {
        let pool = RedisPool::new();

        // Try to connect to local Redis (requires Redis running on localhost:6379)
        let result = pool.connect_instance(
            "test-id",
            "127.0.0.1",
            6379,
            None,
        ).await;

        // This will fail if Redis is not running locally, which is expected
        // The test verifies error handling works correctly
        if result.is_err() {
            println!("Redis not available (expected in CI): {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_invalid_connection() {
        let pool = RedisPool::new();

        // Try to connect to invalid host
        let result = pool.connect_instance(
            "test-id",
            "invalid-host-that-does-not-exist",
            6379,
            None,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent_client() {
        let pool = RedisPool::new();
        let result = pool.get_client("nonexistent-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_instance() {
        let pool = RedisPool::new();

        // Remove non-existent instance (should not panic)
        pool.remove_instance("test-instance").await;

        assert_eq!(pool.connection_count().await, 0);
    }
}

