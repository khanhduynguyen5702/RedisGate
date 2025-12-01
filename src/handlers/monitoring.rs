/// Metrics and monitoring HTTP endpoints
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::middleware::AppState;

/// Health check endpoint
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
    }))
}

/// Readiness check endpoint
pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check database connection
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .is_ok();

    if db_healthy {
        Ok(Json(json!({
            "status": "ready",
            "database": "connected",
            "timestamp": chrono::Utc::now(),
        })))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not_ready",
                "database": "disconnected",
                "timestamp": chrono::Utc::now(),
            })),
        ))
    }
}

/// Get current metrics
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let snapshot = state.metrics.snapshot();

    Json(json!({
        "metrics": {
            "total_requests": snapshot.total_requests,
            "total_success": snapshot.total_success,
            "total_errors": snapshot.total_errors,
            "success_rate": format!("{:.2}%", snapshot.success_rate()),
            "error_rate": format!("{:.2}%", snapshot.error_rate()),
            "redis_commands": snapshot.redis_commands,
            "redis_connection_errors": snapshot.redis_connection_errors,
            "auth_failures": snapshot.auth_failures,
            "uptime_seconds": snapshot.uptime_seconds,
        },
        "timestamp": snapshot.timestamp,
    }))
}

/// Prometheus-compatible metrics endpoint
pub async fn prometheus_metrics(
    State(state): State<Arc<AppState>>,
) -> String {
    let snapshot = state.metrics.snapshot();

    format!(
        "# HELP redisgate_requests_total Total number of requests\n\
         # TYPE redisgate_requests_total counter\n\
         redisgate_requests_total {}\n\
         \n\
         # HELP redisgate_success_total Total number of successful responses\n\
         # TYPE redisgate_success_total counter\n\
         redisgate_success_total {}\n\
         \n\
         # HELP redisgate_errors_total Total number of errors\n\
         # TYPE redisgate_errors_total counter\n\
         redisgate_errors_total {}\n\
         \n\
         # HELP redisgate_redis_commands_total Total number of Redis commands executed\n\
         # TYPE redisgate_redis_commands_total counter\n\
         redisgate_redis_commands_total {}\n\
         \n\
         # HELP redisgate_redis_connection_errors_total Total number of Redis connection errors\n\
         # TYPE redisgate_redis_connection_errors_total counter\n\
         redisgate_redis_connection_errors_total {}\n\
         \n\
         # HELP redisgate_auth_failures_total Total number of authentication failures\n\
         # TYPE redisgate_auth_failures_total counter\n\
         redisgate_auth_failures_total {}\n\
         \n\
         # HELP redisgate_uptime_seconds Server uptime in seconds\n\
         # TYPE redisgate_uptime_seconds gauge\n\
         redisgate_uptime_seconds {}\n",
        snapshot.total_requests,
        snapshot.total_success,
        snapshot.total_errors,
        snapshot.redis_commands,
        snapshot.redis_connection_errors,
        snapshot.auth_failures,
        snapshot.uptime_seconds,
    )
}

