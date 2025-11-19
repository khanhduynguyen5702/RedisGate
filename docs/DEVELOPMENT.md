# 🛠️ Hướng Dẫn Phát Triển - RedisGate

Tài liệu này dành cho developers muốn đóng góp hoặc phát triển RedisGate.

---

## 📋 Môi Trường Development

### Yêu Cầu Hệ Thống:
- **Rust**: 1.70+ (stable)
- **Docker Desktop**: Latest version
- **PostgreSQL Client**: Optional (psql, pgAdmin)
- **Git**: Latest version
- **IDE**: VSCode (recommended) hoặc IntelliJ IDEA

### Recommended VSCode Extensions:
- rust-analyzer
- Better TOML
- Docker
- PostgreSQL

---

## 🚀 Setup Ban Đầu

### 1. Clone Repository:
```bash
git clone https://github.com/yourusername/redisgate.git
cd redisgate
```

### 2. Setup Environment:
```bash
# Copy environment file
cp .env.development .env

# Review và chỉnh sửa nếu cần
cat .env
```

### 3. Install Dependencies:
```bash
# Rust dependencies
cargo build

# Install SQLx CLI (for migrations)
cargo install sqlx-cli --no-default-features --features postgres
```

### 4. Start Services:
```bash
# Start PostgreSQL và Redis
docker-compose up -d

# Verify
docker ps
```

### 5. Setup Database:
```bash
# Migrations sẽ tự động chạy khi start server
cargo run --bin redisgate
```

---

## 📁 Cấu Trúc Dự Án

```
RedisGate/
├── src/
│   ├── main.rs                 # Entry point
│   ├── handlers/               # API route handlers
│   │   ├── auth.rs            # Authentication
│   │   ├── redis.rs           # Redis commands
│   │   ├── redis_instances.rs # Instance management
│   │   ├── api_keys.rs        # API keys
│   │   ├── organizations.rs   # Organizations
│   │   └── quota.rs           # Quota management
│   ├── services/
│   │   └── quota.rs           # Quota service
│   ├── models.rs              # Database models
│   ├── auth.rs                # JWT & auth logic
│   ├── middleware.rs          # Request middleware
│   ├── api_models.rs          # API request/response types
│   └── k8s_service.rs         # Kubernetes integration
│
├── migrations/                 # Database migrations
│   └── *.sql
│
├── public/                    # Frontend files
│   ├── index.html            # Landing page
│   ├── login.html            # Login page
│   └── dashboard.html        # Dashboard
│
├── tests/                     # Integration tests
├── docker-compose.yml        # Docker services
└── Cargo.toml                # Rust dependencies
```

---

## 🔧 Common Development Tasks

### Chạy Server (Development Mode):
```bash
# Chạy với auto-reload (cần cargo-watch)
cargo install cargo-watch
cargo watch -x 'run --bin redisgate'

# Hoặc chạy bình thường
cargo run --bin redisgate
```

### Chạy Tests:
```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests
cargo test --test '*'
```

### Database Migrations:

#### Tạo Migration Mới:
```bash
# Using SQLx CLI
sqlx migrate add create_new_feature

# Edit the file in migrations/
# Then run:
cargo run  # Migrations auto-run on startup
```

#### Reset Database:
```bash
# Drop and recreate
docker-compose down -v
docker-compose up -d
# Wait 5 seconds
cargo run
```

#### Manual Migration:
```bash
# Connect to database
docker exec -it redisgate-postgres psql -U redisgate_dev -d redisgate_dev

# Run SQL
\i migrations/your_migration.sql
```

### Code Quality:

#### Format Code:
```bash
cargo fmt
```

#### Lint Code:
```bash
cargo clippy
```

#### Fix Warnings:
```bash
cargo fix --allow-dirty
```

---

## 🧪 Testing Strategy

### Unit Tests:
```rust
// src/services/quota.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_calculation() {
        // Test code
    }
}
```

### Integration Tests:
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_create_instance() {
    // Setup test database
    // Make API call
    // Assert response
}
```

### Manual Testing:
```bash
# Test connection tool
cargo run --bin test_connections

# Demo JWT
cargo run --bin demo_jwt
```

---

## 📝 Coding Guidelines

### Rust Style:
- Sử dụng `rustfmt` default config
- Follow Rust naming conventions
- Add comments cho complex logic
- Use `Result<T, E>` cho error handling

### API Design:
- RESTful conventions
- JSON responses
- Proper HTTP status codes
- Clear error messages

### Database:
- Luôn dùng prepared statements (SQLx query!)
- Index foreign keys
- Use transactions cho multi-step operations
- Soft delete (deleted_at field)

### Frontend:
- Vanilla JS (no framework để đơn giản)
- Mobile-responsive
- Clear error messages
- Loading states

---

## 🐛 Debugging

### Server Logs:
```bash
# Set log level
export RUST_LOG=debug
cargo run

# Or inline
RUST_LOG=debug cargo run
```

### Database Queries:
```bash
# Enable SQLx query logging
export RUST_LOG=sqlx::query=debug
```

### Common Issues:

#### Compilation Errors:
```bash
# Clear build cache
cargo clean
cargo build
```

#### Database Connection:
```bash
# Check connection string
echo $DATABASE_URL

# Test connection
psql postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev
```

#### Port Already in Use:
```bash
# Find process
netstat -ano | findstr :3000

# Kill process
taskkill /F /PID <pid>
```

---

## 🔐 Security Best Practices

### JWT Tokens:
- Ngắn expiration time (development: 24h, production: 1h)
- Rotate JWT secret thường xuyên
- Validate claims properly

### Passwords:
- Use bcrypt với cost factor >= 10
- Never log passwords
- Enforce strong password policy

### API Keys:
- Generate crypto-secure random keys
- Hash keys in database
- Allow revocation

### Database:
- Use connection pooling
- Prepared statements only
- Least privilege principle
- Regular backups

---

## 📊 Performance Tips

### Database:
```sql
-- Add indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_instances_org ON redis_instances(organization_id);

-- Use EXPLAIN ANALYZE
EXPLAIN ANALYZE SELECT * FROM redis_instances WHERE organization_id = '...';
```

### API:
- Connection pooling (SQLx default: 5 connections)
- Response caching (future)
- Pagination (implemented)
- Lazy loading

### Frontend:
- Minimize API calls
- Cache static assets
- Debounce user input
- Virtual scrolling (future)

---

## 🚀 Deployment

### Build Production:
```bash
# Optimized build
cargo build --release

# Binary at: target/release/redisgate
```

### Docker:
```bash
# Build image
docker build -t redisgate:latest .

# Run
docker run -p 3000:3000 --env-file .env redisgate:latest
```

### Environment Variables (Production):
```bash
DATABASE_URL=postgresql://user:pass@host:5432/db
JWT_SECRET=<strong-random-secret-256-bits>
RUST_LOG=info
APP_PORT=3000
```

---

## 📚 Tài Nguyên Học Tập

### Rust:
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Axum:
- [Axum Docs](https://docs.rs/axum/latest/axum/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)

### SQLx:
- [SQLx Docs](https://docs.rs/sqlx/latest/sqlx/)
- [SQLx Book](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

### PostgreSQL:
- [PostgreSQL Docs](https://www.postgresql.org/docs/)

---

## 🤝 Contributing

### Workflow:
1. Fork repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Make changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Commit: `git commit -m 'Add amazing feature'`
7. Push: `git push origin feature/amazing-feature`
8. Create Pull Request

### Commit Messages:
```
type(scope): subject

body

footer
```

**Types**: feat, fix, docs, style, refactor, test, chore

**Example**:
```
feat(api): add Redis HSET/HGET commands

- Implement HSET handler
- Implement HGET handler
- Add tests
- Update API docs

Closes #123
```

---

## 📞 Hỗ Trợ

- 🐛 **Bugs**: [GitHub Issues](https://github.com/yourusername/redisgate/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/redisgate/discussions)
- 📧 **Email**: dev@redisgate.io

---

**Happy Coding! 🦀**

