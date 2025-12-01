use redis::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, info, warn};

const MAX_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

/// Redis connection pool for managing multiple Redis instances
#[derive(Clone)]
pub struct RedisPool {
    connections: Arc<RwLock<HashMap<String, Client>>>,
}

impl RedisPool {
    /// Create a new Redis connection pool
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to a Redis instance with retry logic and detailed error reporting
    pub async fn connect_instance(
        &self,
        instance_id: &str,
        host: &str,
        port: u16,
        password: Option<&str>,
    ) -> Result<(), String> {
        let redis_url = if let Some(pwd) = password {
            format!("redis://:{}@{}:{}", pwd, host, port)
        } else {
            format!("redis://{}:{}", host, port)
        };

        info!(
            instance_id = %instance_id,
            host = %host,
            port = %port,
            "Attempting to connect to Redis instance"
        );

        // Retry logic
        let mut last_error = String::new();

        for attempt in 1..=MAX_RETRY_ATTEMPTS {
            match Client::open(redis_url.as_str()) {
                Ok(client) => {
                    // Test connection with PING
                    match client.get_connection() {
                        Ok(mut conn) => {
                            // Verify with PING
                            match redis::cmd("PING").query::<String>(&mut conn) {
                                Ok(response) => {
                                    info!(
                                        instance_id = %instance_id,
                                        attempt = %attempt,
                                        response = %response,
                                        "Successfully connected and verified with PING"
                                    );

                                    // Store client in pool
                                    self.connections
                                        .write()
                                        .await
                                        .insert(instance_id.to_string(), client);

                                    return Ok(());
                                }
                                Err(e) => {
                                    last_error = format!("PING failed: {}", e);
                                    warn!(
                                        instance_id = %instance_id,
                                        attempt = %attempt,
                                        error = %e,
                                        "PING command failed"
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            last_error = format!("Connection failed: {}", e);
                            warn!(
                                instance_id = %instance_id,
                                attempt = %attempt,
                                error = %e,
                                "Failed to establish connection"
                            );
                        }
                    }
                }
                Err(e) => {
                    last_error = format!("Invalid URL: {}", e);
                    error!(
                        instance_id = %instance_id,
                        attempt = %attempt,
                        error = %e,
                        "Invalid Redis URL configuration"
                    );
                }
            }

            // Wait before retry (except on last attempt)
            if attempt < MAX_RETRY_ATTEMPTS {
                warn!(
                    instance_id = %instance_id,
                    attempt = %attempt,
                    "Retrying connection in {}ms...",
                    RETRY_DELAY_MS
                );
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        error!(
            instance_id = %instance_id,
            "Failed to connect after {} attempts",
            MAX_RETRY_ATTEMPTS
        );

        Err(format!(
            "Failed to connect after {} attempts. Last error: {}",
            MAX_RETRY_ATTEMPTS, last_error
        ))
    }

    /// Get a client for an instance
    pub async fn get_client(&self, instance_id: &str) -> Result<Client, String> {
        self.connections
            .read()
            .await
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                warn!(instance_id = %instance_id, "No connection found in pool");
                format!("No connection found for instance {}", instance_id)
            })
    }

    /// Health check for a specific instance
    pub async fn health_check(&self, instance_id: &str) -> Result<String, String> {
        let client = self.get_client(instance_id).await?;

        match client.get_connection() {
            Ok(mut conn) => {
                match redis::cmd("PING").query::<String>(&mut conn) {
                    Ok(response) => {
                        info!(instance_id = %instance_id, "Health check passed");
                        Ok(response)
                    }
                    Err(e) => {
                        error!(instance_id = %instance_id, "Health check failed: {}", e);
                        Err(format!("PING failed: {}", e))
                    }
                }
            }
            Err(e) => {
                error!(instance_id = %instance_id, "Failed to get connection: {}", e);
                Err(format!("Connection error: {}", e))
            }
        }
    }

    /// Remove an instance from the pool
    pub async fn remove_instance(&self, instance_id: &str) {
        self.connections.write().await.remove(instance_id);
        info!(instance_id = %instance_id, "Instance removed from pool");
    }

    /// Get count of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Check if an instance exists in the pool
    pub async fn has_instance(&self, instance_id: &str) -> bool {
        self.connections.read().await.contains_key(instance_id)
    }

    /// Reconnect to an instance (useful for health recovery)
    pub async fn reconnect_instance(
        &self,
        instance_id: &str,
        host: &str,
        port: u16,
        password: Option<&str>,
    ) -> Result<(), String> {
        // Remove old connection
        self.remove_instance(instance_id).await;

        // Try to reconnect
        self.connect_instance(instance_id, host, port, password).await
    }
}

impl Default for RedisPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_invalid_connection() {
        let pool = RedisPool::new();

        // Try to connect to invalid host
        let result = pool.connect_instance(
            "test-id",
            "invalid-host-that-does-not-exist.local",
            6379,
            None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection failed"));
    }
}


