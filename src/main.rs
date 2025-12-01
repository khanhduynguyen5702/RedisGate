use axum::{
    extract::{Extension, State},
    middleware as axum_middleware,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api_models;
mod auth;
mod handlers;
pub mod k8s_service;
#[cfg(test)]
mod k8s_tests;
mod middleware;
mod models;
pub mod services;
mod config;
mod monitoring;

use config::Config;

#[tokio::main]
async fn main() {
    // Initialize structured logging with environment-based configuration
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default log levels
            EnvFilter::new("info,redisgate=debug,tower_http=debug,axum=debug")
        });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .compact() // Use compact format for better readability
        )
        .init();

    info!("🚀 RedisGate starting up...");
    debug!("Logging system initialized with structured logging");

    // Load environment variables - prioritize .env.development for development
    if std::path::Path::new(".env.development").exists() {
        dotenv::from_filename(".env.development").ok();
    } else {
        dotenv::dotenv().ok();
    }

    // Load configuration
    let config = match Config::load() {
        Ok(cfg) => {
            info!("✓ Configuration loaded successfully");
            info!("  Server: {}", cfg.bind_address());
            info!("  Rate limit: {} req/s (enabled: {})",
                cfg.rate_limit.default_requests_per_second,
                cfg.rate_limit.enabled);
            info!("  Metrics: {}", if cfg.metrics.enabled { "enabled" } else { "disabled" });
            cfg
        },
        Err(e) => {
            warn!("Failed to load config file: {}. Using environment variables.", e);
            // Fallback to env vars if config file not found
            Config::load().unwrap_or_else(|_| {
                panic!("Could not load configuration from file or environment");
            })
        }
    };

    // Database connection using config
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| config.database.url.clone());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| config.security.jwt_secret.clone());

    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.connection_timeout_seconds))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_seconds))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    info!("✓ Database connected");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database migrations completed successfully");

    // Create application state with config
    let app_state = Arc::new(middleware::AppState::with_config(
        pool.clone(),
        &jwt_secret,
        config.rate_limit.default_requests_per_second,
    ));

    // Build protected API routes with auth middleware
    let protected_api = Router::new()
        .route("/organizations", post(handlers::organizations::create_organization))
        .route("/organizations", get(handlers::organizations::list_organizations))
        .route("/organizations/:org_id", get(handlers::organizations::get_organization))
        .route("/organizations/:org_id", put(handlers::organizations::update_organization))
        .route("/organizations/:org_id", delete(handlers::organizations::delete_organization))
        .route("/organizations/:org_id/quota", get(handlers::quota::get_quota))
        .route("/organizations/:org_id/quota", put(handlers::quota::update_quota))
        .route("/organizations/:org_id/api-keys", post(handlers::api_keys::create_api_key))
        .route("/organizations/:org_id/api-keys", get(handlers::api_keys::list_api_keys))
        .route("/organizations/:org_id/api-keys/:key_id", get(handlers::api_keys::get_api_key))
        .route("/organizations/:org_id/api-keys/:key_id", delete(handlers::api_keys::revoke_api_key))
        .route("/organizations/:org_id/redis-instances", post(handlers::redis_instances::create_redis_instance))
        .route("/organizations/:org_id/redis-instances", get(handlers::redis_instances::list_redis_instances))
        .route("/organizations/:org_id/redis-instances/:instance_id", get(handlers::redis_instances::get_redis_instance))
        .route("/organizations/:org_id/redis-instances/:instance_id/status", put(handlers::redis_instances::update_redis_instance_status))
        .route("/organizations/:org_id/redis-instances/:instance_id", delete(handlers::redis_instances::delete_redis_instance))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    // Protected auth routes
    let protected_auth = Router::new()
        .route("/me", get(handlers::auth::get_current_user))
        .with_state(app_state.clone())
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    // Build application with all routes
    let app = Router::new()
        // Frontend UI
        .route("/", get(serve_ui))
        .route("/login.html", get(serve_login))
        .route("/dashboard.html", get(serve_dashboard))
        .route("/metrics-dashboard.html", get(serve_metrics_dashboard))
        .route("/debug.html", get(serve_debug))
        .route("/fix.html", get(serve_fix))
        .route("/token-debug.html", get(serve_token_debug))
        .route("/token-check.html", get(serve_token_check))
        .route("/get-api-key.html", get(serve_get_api_key))

        // Health & Monitoring endpoints (no authentication)
        .route("/health", get(services::health::health))
        .route("/health/live", get(services::health::liveness))
        .route("/health/ready", get(services::health::readiness))
        .route("/metrics", get(metrics_handler))
        .route("/monitoring/health", get(handlers::monitoring::health_check))
        .route("/monitoring/ready", get(handlers::monitoring::readiness_check))
        .route("/monitoring/metrics", get(handlers::monitoring::get_metrics))
        .route("/monitoring/prometheus", get(handlers::monitoring::prometheus_metrics))

        // Legacy endpoints
        .route("/version", get(version))
        .route("/stats", get(database_stats))

        // Public routes (no authentication required)
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))

        // Protected auth routes
        .nest("/auth", protected_auth)
        // Protected API routes
        .nest("/api", protected_api)

        // Redis HTTP API routes (uses API key authentication)
        .route("/redis/:instance_id/ping", get(handlers::redis::handle_ping))
        .route("/redis/:instance_id/set/:key/:value", get(handlers::redis::handle_set))
        .route("/redis/:instance_id/get/:key", get(handlers::redis::handle_get))
        .route("/redis/:instance_id/del/:key", get(handlers::redis::handle_del))
        .route("/redis/:instance_id/incr/:key", get(handlers::redis::handle_incr))
        .route("/redis/:instance_id/hset/:key/:field/:value", get(handlers::redis::handle_hset))
        .route("/redis/:instance_id/hget/:key/:field", get(handlers::redis::handle_hget))
        .route("/redis/:instance_id/lpush/:key/:value", get(handlers::redis::handle_lpush))
        .route("/redis/:instance_id/lpop/:key", get(handlers::redis::handle_lpop))
        .route("/redis/:instance_id/expire/:key/:seconds", get(handlers::redis::handle_expire))
        .route("/redis/:instance_id/ttl/:key", get(handlers::redis::handle_ttl))
        .route("/redis/:instance_id/exists/:key", get(handlers::redis::handle_exists))
        .route("/redis/:instance_id/decr/:key", get(handlers::redis::handle_decr))

        // Set operations
        .route("/redis/:instance_id/sadd/:key/:member", get(handlers::redis::handle_sadd))
        .route("/redis/:instance_id/smembers/:key", get(handlers::redis::handle_smembers))
        .route("/redis/:instance_id/sismember/:key/:member", get(handlers::redis::handle_sismember))
        .route("/redis/:instance_id/srem/:key/:member", get(handlers::redis::handle_srem))

        // Generic Redis command endpoint (for POST with JSON body)
        .route("/redis/:instance_id", post(handlers::redis::handle_generic_command))

        // Catch-all route for debugging Redis requests
        .route("/redis/:instance_id/*path", get(handlers::redis::handle_debug_request))
        .layer(CorsLayer::permissive())
        // Metrics tracking middleware (must be early in chain)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::metrics_tracking_middleware,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        status = %response.status(),
                        latency = ?latency,
                        "response generated"
                    );
                })
        )
        .with_state(app_state)
        .layer(Extension(Arc::new(pool)));

    // Start server using config
    let bind_addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| config.bind_address());

    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

   let display_host = if config.server.host == "0.0.0.0" {
       "localhost"
   } else {
       &config.server.host
   };

    info!("🚀 Server running on http://{}:{}", display_host, config.server.port);
    info!("📊 Metrics available at http://{}:{}/metrics", display_host, config.server.port);
    info!("❤️  Health check at http://{}:{}/health", display_host, config.server.port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn serve_ui() -> impl IntoResponse {
    let html = include_str!("../public/index.html");
    Html(html)
}

async fn serve_login() -> impl IntoResponse {
    let html = include_str!("../public/login.html");
    Html(html)
}

async fn serve_dashboard() -> impl IntoResponse {
    let html = include_str!("../public/dashboard.html");
    Html(html)
}

async fn serve_metrics_dashboard() -> impl IntoResponse {
    let html = include_str!("../public/metrics-dashboard.html");
    Html(html)
}

async fn serve_debug() -> impl IntoResponse {
    let html = include_str!("../public/debug.html");
    Html(html)
}

async fn serve_fix() -> impl IntoResponse {
    let html = include_str!("../public/fix.html");
    Html(html)
}

async fn serve_token_debug() -> impl IntoResponse {
    let html = include_str!("../public/token-debug.html");
    Html(html)
}

async fn serve_token_check() -> impl IntoResponse {
    let html = include_str!("../public/token-check.html");
    Html(html)
}

async fn serve_get_api_key() -> impl IntoResponse {
    let html = include_str!("../public/get-api-key.html");
    Html(html)
}

async fn health_check(Extension(pool): Extension<Arc<PgPool>>) -> axum::response::Json<serde_json::Value> {
    // Test database connection
    let db_status = match sqlx::query("SELECT 1 as status")
        .fetch_one(pool.as_ref())
        .await
    {
        Ok(row) => {
            let status: i32 = row.get("status");
            if status == 1 {
                "healthy"
            } else {
                "unhealthy"
            }
        }
        Err(e) => {
            warn!("Database health check failed: {}", e);
            "unhealthy"
        }
    };

    axum::response::Json(json!({
        "status": "ok",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    }))
}

async fn version() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(json!({
        "name": "redisgate",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Cloud Redis on Kubernetes HTTP Gateway"
    }))
}

/// Prometheus metrics endpoint
async fn metrics_handler(
    State(state): State<Arc<middleware::AppState>>,
) -> impl IntoResponse {
    // Update instance count metrics
    if let Ok(count) = sqlx::query!("SELECT COUNT(*) as count FROM redis_instances WHERE deleted_at IS NULL")
        .fetch_one(&state.db_pool)
        .await
    {
        if let Some(count) = count.count {
            services::metrics::MetricsService::set_instance_count(count);
        }
    }

    // Update connection count
    let conn_count = state.redis_pool.connection_count().await;
    services::metrics::MetricsService::set_redis_connections(conn_count as i64);

    // Render metrics
    let metrics = state.metrics_service.render();
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        metrics,
    )
}

async fn database_stats(Extension(pool): Extension<Arc<PgPool>>) -> axum::response::Json<serde_json::Value> {
    // Get table counts to demonstrate compile-time checked queries
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let org_count = sqlx::query!("SELECT COUNT(*) as count FROM organizations")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let redis_instance_count = sqlx::query!("SELECT COUNT(*) as count FROM redis_instances")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let api_key_count = sqlx::query!("SELECT COUNT(*) as count FROM api_keys")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    axum::response::Json(json!({
        "tables": {
            "users": user_count,
            "organizations": org_count,
            "redis_instances": redis_instance_count,
            "api_keys": api_key_count
        },
        "timestamp": chrono::Utc::now()
    }))
}
