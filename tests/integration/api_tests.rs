// Integration tests for RedisGate API

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use std::env;

// Helper function to setup test database
async fn setup_test_db() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://redisgate_dev:devpassword123@localhost:5432/redisgate_dev".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_database_connection() {
    let pool = setup_test_db().await;

    // Test basic query
    let result = sqlx::query!("SELECT 1 as num")
        .fetch_one(&pool)
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().num, Some(1));
}

#[tokio::test]
async fn test_users_table_exists() {
    let pool = setup_test_db().await;

    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = 'users'"
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().count.unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_organizations_table_exists() {
    let pool = setup_test_db().await;

    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = 'organizations'"
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().count.unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_redis_instances_table_exists() {
    let pool = setup_test_db().await;

    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = 'redis_instances'"
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().count.unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_api_keys_table_exists() {
    let pool = setup_test_db().await;

    let result = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = 'api_keys'"
    )
    .fetch_one(&pool)
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap().count.unwrap_or(0) > 0);
}

// Test JWT token generation and verification
#[test]
fn test_jwt_creation() {
    use redisgate::auth::JwtManager;
    use uuid::Uuid;

    let jwt_manager = JwtManager::new("test_secret_key_for_testing".to_string());
    let user_id = Uuid::new_v4();

    let token = jwt_manager.create_token(user_id);
    assert!(token.is_ok());

    let token_string = token.unwrap();
    assert!(!token_string.is_empty());
}

#[test]
fn test_jwt_verification() {
    use redisgate::auth::JwtManager;
    use uuid::Uuid;

    let jwt_manager = JwtManager::new("test_secret_key_for_testing".to_string());
    let user_id = Uuid::new_v4();

    let token = jwt_manager.create_token(user_id).unwrap();
    let verified = jwt_manager.verify_token(&token);

    assert!(verified.is_ok());
    let claims = verified.unwrap();
    assert_eq!(claims.claims.user_id, user_id);
}

#[test]
fn test_jwt_invalid_token() {
    use redisgate::auth::JwtManager;

    let jwt_manager = JwtManager::new("test_secret_key_for_testing".to_string());
    let verified = jwt_manager.verify_token("invalid.token.here");

    assert!(verified.is_err());
}

// Test password hashing
#[test]
fn test_password_hashing() {
    use bcrypt::{hash, verify};

    let password = "test_password_123";
    let hashed = hash(password, 4).expect("Failed to hash password");

    assert_ne!(password, hashed);
    assert!(verify(password, &hashed).unwrap());
    assert!(!verify("wrong_password", &hashed).unwrap());
}

// Test Redis pool
#[tokio::test]
async fn test_redis_pool_basic_operations() {
    use redisgate::services::redis_pool::RedisPool;

    let pool = RedisPool::new();

    // Test initial state
    assert_eq!(pool.connection_count().await, 0);
    assert!(!pool.has_instance("test-id").await);

    // Test get non-existent client
    let result = pool.get_client("non-existent").await;
    assert!(result.is_err());

    // Test remove non-existent instance
    pool.remove_instance("non-existent").await;
    assert_eq!(pool.connection_count().await, 0);
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored (requires actual Redis)
async fn test_redis_connection_to_localhost() {
    use redisgate::services::redis_pool::RedisPool;

    let pool = RedisPool::new();

    // Try connecting to localhost Redis
    let result = pool.connect_instance(
        "test-instance",
        "127.0.0.1",
        6379,
        None,
    ).await;

    if result.is_ok() {
        assert!(pool.has_instance("test-instance").await);
        assert_eq!(pool.connection_count().await, 1);

        // Clean up
        pool.remove_instance("test-instance").await;
        assert_eq!(pool.connection_count().await, 0);
    } else {
        println!("Redis not available on localhost (expected in CI)");
    }
}

