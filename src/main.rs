use axum::{
    extract::Extension,
    http::StatusCode,
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
use tracing::{info, warn};

mod api_models;
mod auth;
mod handlers;
pub mod k8s_service;
#[cfg(test)]
mod k8s_tests;
mod middleware;
mod models;
mod services;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables - prioritize .env.development for development
    if std::path::Path::new(".env.development").exists() {
        dotenv::from_filename(".env.development").ok();
    } else {
        dotenv::dotenv().ok();
    }

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());

    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database migrations completed successfully");

    // Create application state
    let app_state = Arc::new(middleware::AppState::new(pool.clone(), &jwt_secret));

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
        .route("/debug.html", get(serve_debug))
        .route("/fix.html", get(serve_fix))
        // Public routes (no authentication required)
        .route("/health", get(health_check))
        .route("/version", get(version))
        .route("/stats", get(database_stats))
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

        // Generic Redis command endpoint (for POST with JSON body)
        .route("/redis/:instance_id", post(handlers::redis::handle_generic_command))

        // Catch-all route for debugging Redis requests
        .route("/redis/:instance_id/*path", get(handlers::redis::handle_debug_request))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
        .layer(Extension(Arc::new(pool)));

    // Start server
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

    info!("Server starting on {}", bind_addr);
    info!("UI available at http://localhost:{}", port);

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

async fn serve_debug() -> impl IntoResponse {
    let html = include_str!("../public/debug.html");
    Html(html)
}

async fn serve_fix() -> impl IntoResponse {
    let html = include_str!("../public/fix.html");
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
