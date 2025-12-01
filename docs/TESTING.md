# Testing Guide

## Overview

RedisGate includes comprehensive test coverage across multiple levels:

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test API endpoints and database interactions
- **End-to-End Tests**: Test complete user workflows

## Running Tests

### Quick Test

Run all tests:
```bash
cargo test
```

### Run specific test suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific test file
cargo test --test api_tests

# Run ignored tests (requires actual Redis/DB)
cargo test -- --ignored

# Run with output
cargo test -- --nocapture
```

### Test with coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## Test Structure

```
tests/
├── integration/
│   ├── mod.rs
│   └── api_tests.rs          # API integration tests
src/
├── services/
│   └── redis_pool.rs         # Contains unit tests
└── bin/
    └── test_connections.rs   # Manual connection testing tool
```

## Unit Tests

Located within source files using `#[cfg(test)]` modules.

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = some_async_function().await;
        assert!(result.is_ok());
    }
}
```

## Integration Tests

Located in `tests/integration/` directory.

### Database Tests

Require `DATABASE_URL` environment variable:

```bash
export DATABASE_URL="postgres://redisgate_dev:devpassword123@localhost:5432/redisgate_dev"
cargo test test_database_connection
```

### Redis Connection Tests

Some tests are marked with `#[ignore]` and require a running Redis instance:

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run ignored tests
cargo test -- --ignored
```

## Manual Testing Tools

### Test Connections Tool

Verify Redis instance connections:

```bash
cargo run --bin test_connections
```

Output example:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 RedisGate - Instance Connection Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Connecting to database...
   ✓ Database connected successfully

2. Fetching Redis instances from database...
   ✓ Found 3 instance(s)

3. Testing connections to Redis instances...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Instance 1/3
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Name:     My Redis Instance
  ID:       123e4567-e89b-12d3-a456-426614174000
  Host:     localhost:6379
  Status:   "running"
  Org ID:   org-id-here
  Password: ✓ Set

  Testing connection... ✅ SUCCESS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Total instances:  3
  ✅ Successful:     2
  ❌ Failed:         1
```

## CI/CD Testing

GitHub Actions automatically runs tests on every push/PR.

### Workflow

1. **Lint**: Check code formatting and run clippy
2. **Security Audit**: Check for security vulnerabilities
3. **Test**: Run all tests with PostgreSQL and Redis services
4. **Build**: Compile release binary

See `.github/workflows/ci.yml` for configuration.

### Local CI simulation

```bash
# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo install cargo-audit
cargo audit

# Full test suite
cargo test --all
```

## Test Database Setup

For integration tests, you need a test database:

```bash
# Start PostgreSQL
docker run -d \
  -p 5432:5432 \
  -e POSTGRES_USER=redisgate_dev \
  -e POSTGRES_PASSWORD=devpassword123 \
  -e POSTGRES_DB=redisgate_dev \
  postgres:15

# Run migrations
export DATABASE_URL="postgres://redisgate_dev:devpassword123@localhost:5432/redisgate_dev"
sqlx migrate run

# Run tests
cargo test
```

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Arrange
        let input = "test";
        
        // Act
        let result = my_function(input);
        
        // Assert
        assert_eq!(result, "expected");
    }

    #[tokio::test]
    async fn test_async_function() {
        // Arrange
        let state = setup_test_state().await;
        
        // Act
        let result = async_function(&state).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Test Template

```rust
// tests/integration/my_feature_test.rs

use sqlx::PgPool;

async fn setup() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

#[tokio::test]
async fn test_my_feature() {
    let pool = setup().await;
    
    // Test logic here
    
    // Cleanup if needed
}
```

## Test Coverage Goals

- **Unit Tests**: 80%+ coverage
- **Integration Tests**: All API endpoints
- **Critical Paths**: 100% coverage (auth, Redis operations)

## Current Coverage

Run to check:
```bash
cargo tarpaulin --out Stdout
```

Target: **70%+ overall coverage**

## Troubleshooting

### Tests fail with "Database error"

Ensure PostgreSQL is running and migrations are applied:
```bash
docker-compose up -d postgres
sqlx migrate run
```

### Tests fail with "Redis connection error"

For tests requiring Redis (marked `#[ignore]`):
```bash
docker-compose up -d redis
cargo test -- --ignored
```

### SQLx compile-time checks fail

Regenerate SQLx metadata:
```bash
cargo sqlx prepare
```

## Best Practices

1. **Isolate tests**: Each test should be independent
2. **Clean up**: Remove test data after tests complete
3. **Use fixtures**: Create reusable test data helpers
4. **Mock external services**: Don't depend on external APIs in tests
5. **Test edge cases**: Not just happy paths
6. **Descriptive names**: `test_user_registration_with_invalid_email`
7. **Fast tests**: Keep unit tests under 100ms

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [SQLx Testing](https://github.com/launchbadge/sqlx#testing)

