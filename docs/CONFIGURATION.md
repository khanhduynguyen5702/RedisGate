# Configuration Management Guide

## ✅ Overview

RedisGate now uses a **comprehensive configuration system** that supports:

- ✅ TOML configuration files
- ✅ Environment variable overrides
- ✅ Multiple environments (dev, test, production)
- ✅ Validation and error handling
- ✅ Type-safe configuration

---

## 📁 Configuration Files

### Development
```bash
config.toml  # Default development config
```

### Production
```bash
config.production.toml  # Production-optimized settings
```

### Testing
```bash
config.test.toml  # Test environment settings
```

---

## 🚀 Quick Start

### 1. Copy environment template
```bash
cp .env.example .env
```

### 2. Edit configuration
```bash
# Edit config.toml for development
# OR
# Set environment variables in .env
```

### 3. Run application
```bash
cargo run --bin redisgate
```

The app will:
1. Load `config.toml` (or `config.production.toml` if `ENVIRONMENT=production`)
2. Override with environment variables from `.env`
3. Validate all settings
4. Start with configured values

---

## ⚙️ Configuration Structure

### Server Configuration
```toml
[server]
host = "0.0.0.0"              # Bind address
port = 3000                    # HTTP port
workers = 4                    # Worker threads (auto-detect if omitted)
request_timeout_seconds = 30   # Request timeout
max_request_size_mb = 10      # Max request body size
```

**Environment overrides:**
- `SERVER_HOST` - Override bind address
- `SERVER_PORT` - Override port

### Database Configuration
```toml
[database]
url = "postgresql://user:pass@host:5432/db"  # Connection string
max_connections = 10           # Max pool size
min_connections = 2            # Min pool size
connection_timeout_seconds = 30
idle_timeout_seconds = 600
enable_logging = true          # Log SQL queries
```

**Environment overrides:**
- `DATABASE_URL` - Override database URL (recommended)

### Redis Configuration
```toml
[redis]
default_timeout_ms = 5000      # Default command timeout
max_retries = 3                # Connection retry attempts
retry_delay_ms = 1000          # Delay between retries
pool_size = 10                 # Connection pool size
```

### Rate Limiting
```toml
[rate_limit]
default_requests_per_second = 100  # Default RPS limit
burst_size = 20                     # Burst allowance
enabled = true                      # Enable/disable rate limiting
```

**Environment overrides:**
- `RATE_LIMIT_RPS` - Override default RPS

### Metrics
```toml
[metrics]
enabled = true                 # Enable Prometheus metrics
path = "/metrics"              # Metrics endpoint path
histogram_buckets = [0.001, 0.005, ...]  # Latency buckets
```

### Health Checks
```toml
[health]
enabled = true                 # Enable health endpoints
check_interval_seconds = 30    # Background check interval
```

### Security
```toml
[security]
jwt_secret = "your-secret-key-min-32-chars"  # JWT signing key
token_expiry_hours = 24        # User token expiry
api_key_expiry_days = 365      # API key expiry
enable_https = false           # HTTPS support
enable_cors = true             # CORS support
cors_allowed_origins = [...]   # Allowed origins
```

**Environment overrides:**
- `JWT_SECRET` - Override JWT secret (REQUIRED in production)

### Logging
```toml
[logging]
level = "info"                 # trace, debug, info, warn, error
json_format = false            # JSON structured logs
log_to_file = false            # Write to file
log_file_path = "logs/app.log"
```

**Environment overrides:**
- `RUST_LOG` - Override log level

---

## 🔧 Environment Variables

### Priority Order
1. Environment variables (highest priority)
2. `.env` file
3. `config.toml` file (lowest priority)

### Required Variables

**Development:**
```bash
DATABASE_URL=postgresql://user:pass@localhost/db
JWT_SECRET=your-secret-key-minimum-32-characters
```

**Production:**
```bash
ENVIRONMENT=production
DATABASE_URL=postgresql://user:pass@host/db
JWT_SECRET=super-secret-production-key-minimum-32-chars
```

### Optional Overrides

```bash
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Rate limiting
RATE_LIMIT_RPS=500

# Config file selection
CONFIG_PATH=config.production.toml

# Logging
RUST_LOG=debug
```

---

## 📋 Environment Selection

### Auto-detection
```bash
# Development (default)
cargo run

# Production
ENVIRONMENT=production cargo run

# Testing
ENVIRONMENT=test cargo test
```

### Manual selection
```bash
# Use specific config file
CONFIG_PATH=config.production.toml cargo run
```

---

## ✅ Validation

Configuration is validated on startup:

```rust
✓ Server port cannot be 0
✓ Workers must be >= 1
✓ Database URL is required
✓ Max connections > 0
✓ Min connections <= max connections
✓ JWT secret >= 32 characters (warning if shorter)
✓ Rate limit RPS > 0 (if enabled)
```

**On validation failure:**
```
Error: Invalid configuration: Server port cannot be 0
```

---

## 🔐 Security Best Practices

### 1. Never commit secrets
```bash
# .gitignore should include:
.env
.env.*
!.env.example
config.production.toml  # If contains secrets
```

### 2. Use environment variables in production
```bash
# Don't hardcode in config files
DATABASE_URL=...  # From environment
JWT_SECRET=...    # From environment
```

### 3. Rotate secrets regularly
```bash
# Change JWT_SECRET periodically
# Update DATABASE_URL passwords
# Regenerate API keys
```

### 4. Use strong secrets
```bash
# Generate secure JWT secret:
openssl rand -base64 48

# Or:
python3 -c "import secrets; print(secrets.token_urlsafe(48))"
```

---

## 🧪 Testing Configuration

### Unit Tests
```rust
#[test]
fn test_config_validation() {
    let config = Config::from_file("config.test.toml").unwrap();
    assert!(config.validate().is_ok());
}
```

### Load Test Config
```rust
#[tokio::test]
async fn test_with_config() {
    let config = Config::from_file("config.test.toml").unwrap();
    // Use config in tests
}
```

---

## 📊 Configuration Examples

### Development (High Verbosity)
```toml
[logging]
level = "debug"
json_format = false
log_to_file = true

[database]
enable_logging = true

[metrics]
enabled = true

[rate_limit]
enabled = false  # Disable for local dev
```

### Production (Optimized)
```toml
[server]
workers = 16  # Scale with CPU

[logging]
level = "warn"
json_format = true  # Structured logs

[database]
max_connections = 100
enable_logging = false  # Performance

[rate_limit]
enabled = true
default_requests_per_second = 1000
```

### Testing (Fast & Isolated)
```toml
[server]
port = 0  # Random port

[rate_limit]
enabled = false  # No rate limiting

[metrics]
enabled = false  # Faster tests

[database]
max_connections = 5
```

---

## 🐛 Troubleshooting

### Issue: Config file not found
```
Error: Failed to load configuration: No such file or directory
```

**Solution:**
```bash
# Check file exists
ls config.toml

# Or use environment variables
export DATABASE_URL=...
export JWT_SECRET=...
cargo run
```

### Issue: Validation failed
```
Error: Invalid configuration: JWT secret is required
```

**Solution:**
```toml
# Add to config.toml:
[security]
jwt_secret = "your-secret-key-minimum-32-characters-long"
```

### Issue: Environment override not working
```bash
# Check variable name (case-sensitive)
SERVER_PORT=8080  # ✓ Correct
server_port=8080  # ✗ Wrong
```

---

## 📝 Migration from Old Setup

### Before (environment only)
```bash
# .env
DATABASE_URL=postgresql://...
JWT_SECRET=secret
APP_PORT=3000
```

### After (config + env)
```toml
# config.toml
[server]
port = 3000

[database]
url = "postgresql://..."  # Can be overridden by DATABASE_URL

[security]
jwt_secret = "secret"  # Can be overridden by JWT_SECRET
```

**Benefits:**
- ✅ Type-safe configuration
- ✅ Validation on startup
- ✅ Better organization
- ✅ Multiple environment support
- ✅ Self-documenting

---

## 🚀 Deployment

### Docker
```dockerfile
# Dockerfile
COPY config.production.toml /app/config.toml
ENV ENVIRONMENT=production
ENV DATABASE_URL=postgresql://...
ENV JWT_SECRET=...
```

### Kubernetes
```yaml
# ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: redisgate-config
data:
  config.toml: |
    [server]
    port = 3000
    ...

---
# Secret
apiVersion: v1
kind: Secret
metadata:
  name: redisgate-secrets
type: Opaque
data:
  DATABASE_URL: <base64>
  JWT_SECRET: <base64>
```

### Systemd
```ini
[Service]
Environment="ENVIRONMENT=production"
Environment="DATABASE_URL=postgresql://..."
Environment="JWT_SECRET=..."
ExecStart=/usr/local/bin/redisgate
```

---

## 📚 API Reference

### Config::load()
```rust
// Load config from file (auto-detected environment)
let config = Config::load()?;
```

### Config::from_file()
```rust
// Load from specific file
let config = Config::from_file("config.toml")?;
```

### Config::from_file_with_env()
```rust
// Load with environment overrides
let config = Config::from_file_with_env("config.toml")?;
```

### Config::validate()
```rust
// Validate configuration
config.validate()?;
```

---

## ✅ Checklist

- [x] Configuration module implemented
- [x] Default config file (config.toml)
- [x] Production config (config.production.toml)
- [x] Test config (config.test.toml)
- [x] Environment variable overrides
- [x] Validation with error messages
- [x] Environment selection (dev/prod/test)
- [x] Documentation complete
- [x] Examples provided
- [x] Security best practices documented

---

**Status:** ✅ Config Management Complete  
**Impact:** +2% progress (87% total)  
**Next:** Integration tests & API key expiration

---

**Last Updated:** November 24, 2025

