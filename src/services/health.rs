use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

use crate::services::redis_pool::RedisPool;

/// Health check status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Overall health response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub components: HealthComponents,
}

/// Health of individual components
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthComponents {
    pub database: ComponentHealth,
    pub redis_pool: ComponentHealth,
}

/// Health check service
pub struct HealthCheckService {
    start_time: Instant,
}

impl HealthCheckService {
    pub fn new() -> Self {
        info!("Health check service initialized");
        Self {
            start_time: Instant::now(),
        }
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Check database health
    pub async fn check_database(pool: &PgPool) -> ComponentHealth {
        let start = Instant::now();

        match sqlx::query!("SELECT 1 as health_check")
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                let elapsed = start.elapsed().as_millis() as u64;
                ComponentHealth {
                    status: HealthStatus::Healthy,
                    message: Some("Database connection OK".to_string()),
                    response_time_ms: Some(elapsed),
                }
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                ComponentHealth {
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Database error: {}", e)),
                    response_time_ms: None,
                }
            }
        }
    }

    /// Check Redis pool health
    pub async fn check_redis_pool(pool: &RedisPool) -> ComponentHealth {
        let count = pool.connection_count().await;

        ComponentHealth {
            status: if count > 0 {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            },
            message: Some(format!("{} active connections", count)),
            response_time_ms: Some(0),
        }
    }

    /// Perform full health check
    pub async fn check_health(
        &self,
        db_pool: &PgPool,
        redis_pool: &RedisPool,
    ) -> HealthResponse {
        let db_health = Self::check_database(db_pool).await;
        let redis_health = Self::check_redis_pool(redis_pool).await;

        // Overall status is unhealthy if any component is unhealthy
        let overall_status = if db_health.status == HealthStatus::Unhealthy
            || redis_health.status == HealthStatus::Unhealthy
        {
            HealthStatus::Unhealthy
        } else if db_health.status == HealthStatus::Degraded
            || redis_health.status == HealthStatus::Degraded
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthResponse {
            status: overall_status,
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.uptime_seconds(),
            components: HealthComponents {
                database: db_health,
                redis_pool: redis_health,
            },
        }
    }
}

impl Default for HealthCheckService {
    fn default() -> Self {
        Self::new()
    }
}

/// Liveness probe endpoint - returns 200 if service is running
pub async fn liveness() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe endpoint - returns 200 if service is ready to accept traffic
pub async fn readiness(
    State(state): State<Arc<crate::middleware::AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Quick database check
    match sqlx::query!("SELECT 1 as ready").fetch_one(&state.db_pool).await {
        Ok(_) => Ok(Json(json!({
            "status": "ready",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Full health check endpoint
pub async fn health(
    State(state): State<Arc<crate::middleware::AppState>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let health_response = state
        .health_service
        .check_health(&state.db_pool, &state.redis_pool)
        .await;

    let status_code = match health_response.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still OK but degraded
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    if status_code != StatusCode::OK && health_response.status == HealthStatus::Unhealthy {
        Err(status_code)
    } else {
        Ok(Json(health_response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_service_creation() {
        let service = HealthCheckService::new();
        assert!(service.uptime_seconds() < 1);
    }

    #[test]
    fn test_uptime_tracking() {
        let service = HealthCheckService::new();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(service.uptime_seconds() < 10);
    }

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some("OK".to_string()),
            response_time_ms: Some(10),
        };

        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, Some("OK".to_string()));
        assert_eq!(health.response_time_ms, Some(10));
    }

    #[test]
    fn test_health_status_serialization() {
        let healthy = serde_json::to_string(&HealthStatus::Healthy).unwrap();
        assert_eq!(healthy, "\"healthy\"");

        let degraded = serde_json::to_string(&HealthStatus::Degraded).unwrap();
        assert_eq!(degraded, "\"degraded\"");

        let unhealthy = serde_json::to_string(&HealthStatus::Unhealthy).unwrap();
        assert_eq!(unhealthy, "\"unhealthy\"");
    }
}

