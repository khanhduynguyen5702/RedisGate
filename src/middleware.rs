// Authentication middleware for protecting routes

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::{AuthError, JwtManager};
use crate::models::User;
use crate::monitoring::Metrics;

// Middleware for JWT authentication
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    tracing::info!("Auth header: {:?}", auth_header);

    let token = auth_header
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("Missing or invalid Authorization header");
            AuthError::MissingToken
        })?;

    tracing::info!("Token received: {}...", &token[..20.min(token.len())]);

    let token_data = state.jwt_manager.verify_token(token).map_err(|e| {
        tracing::error!("Token verification failed: {:?}", e);
        e
    })?;

    let claims = &token_data.claims;

    // Verify user still exists and is active
    tracing::info!("Verifying user: {}", claims.user_id);

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND is_active = true",
        claims.user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error while fetching user: {}", e);
        AuthError::UserNotFound
    })?
    .ok_or_else(|| {
        tracing::warn!("User not found or not active: {}", claims.user_id);
        AuthError::UserNotFound
    })?;

    if !user.is_active.unwrap_or(true) {
        tracing::warn!("User account is not active: {}", user.id);
        return Err(AuthError::UserNotActive);
    }

    tracing::info!("User authenticated successfully: {}", user.email);

    // Store user info in request extensions for handlers to use
    request.extensions_mut().insert(CurrentUser {
        id: user.id,
        email: user.email.clone(),
        username: user.username.clone(),
        org_id: claims.org_id,
    });

    Ok(next.run(request).await)
}

// Current user info extracted from JWT
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: String,
    pub org_id: Option<uuid::Uuid>,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_manager: JwtManager,
    pub redis_pool: crate::services::redis_pool::RedisPool,
    pub metrics_service: Arc<crate::services::metrics::MetricsService>,
    pub rate_limiter: Arc<crate::services::rate_limiter::RateLimiter>,
    pub health_service: Arc<crate::services::health::HealthCheckService>,
    pub metrics: Metrics,
}

impl AppState {
    pub fn new(db_pool: PgPool, jwt_secret: &str) -> Self {
        // Use default rate limit if no config provided
        Self::with_config(db_pool, jwt_secret, 100)
    }

    pub fn with_config(db_pool: PgPool, jwt_secret: &str, rate_limit_rps: u32) -> Self {
        Self {
            db_pool,
            jwt_manager: JwtManager::new(jwt_secret),
            redis_pool: crate::services::redis_pool::RedisPool::new(),
            metrics_service: Arc::new(crate::services::metrics::MetricsService::new()),
            rate_limiter: Arc::new(crate::services::rate_limiter::RateLimiter::new(rate_limit_rps)),
            health_service: Arc::new(crate::services::health::HealthCheckService::new()),
            metrics: Metrics::new(),
        }
    }
}

/// Metrics tracking middleware - tracks all requests automatically
pub async fn metrics_tracking_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    use std::time::Instant;
    use tracing::{error, info};

    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Increment total requests counter
    state.metrics.inc_requests();

    // Track Redis commands
    if path.starts_with("/redis/") {
        state.metrics.inc_redis_commands();
    }

    // Process the request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();

    // Track response based on status
    let status = response.status();
    if status.is_success() {
        state.metrics.inc_success();
        info!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request succeeded"
        );
    } else if status.is_client_error() || status.is_server_error() {
        state.metrics.inc_errors();

        // Track auth failures specifically
        if status == axum::http::StatusCode::UNAUTHORIZED {
            state.metrics.inc_auth_failures();
        }

        error!(
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request failed"
        );
    }

    response
}
