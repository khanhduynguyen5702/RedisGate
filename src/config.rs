use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Invalid configuration: {0}")]
    Validation(String),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub rate_limit: RateLimitConfig,
    pub metrics: MetricsConfig,
    pub health: HealthConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_workers")]
    pub workers: usize,

    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,

    #[serde(default = "default_max_request_size")]
    pub max_request_size_mb: usize,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_seconds: u64,

    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: u64,

    #[serde(default)]
    pub enable_logging: bool,
}

/// Redis pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_timeout")]
    pub default_timeout_ms: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_rate_limit_rps")]
    pub default_requests_per_second: u32,

    #[serde(default = "default_burst_size")]
    pub burst_size: u32,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_metrics_path")]
    pub path: String,

    #[serde(default = "default_histogram_buckets")]
    pub histogram_buckets: Vec<f64>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_check_interval")]
    pub check_interval_seconds: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,

    #[serde(default = "default_token_expiry")]
    pub token_expiry_hours: u64,

    #[serde(default = "default_api_key_expiry")]
    pub api_key_expiry_days: u64,

    #[serde(default)]
    pub enable_https: bool,

    #[serde(default = "default_enabled")]
    pub enable_cors: bool,

    #[serde(default = "default_cors_origins")]
    pub cors_allowed_origins: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub json_format: bool,

    #[serde(default)]
    pub log_to_file: bool,

    #[serde(default = "default_log_file")]
    pub log_file_path: String,
}

// Default value functions
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 3000 }
fn default_workers() -> usize { num_cpus::get() }
fn default_request_timeout() -> u64 { 30 }
fn default_max_request_size() -> usize { 10 }

fn default_max_connections() -> u32 { 10 }
fn default_min_connections() -> u32 { 2 }
fn default_connection_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }

fn default_redis_timeout() -> u64 { 5000 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_delay() -> u64 { 1000 }
fn default_pool_size() -> usize { 10 }

fn default_rate_limit_rps() -> u32 { 100 }
fn default_burst_size() -> u32 { 20 }

fn default_enabled() -> bool { true }
fn default_metrics_path() -> String { "/metrics".to_string() }
fn default_histogram_buckets() -> Vec<f64> {
    vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
}

fn default_check_interval() -> u64 { 30 }

fn default_token_expiry() -> u64 { 24 }
fn default_api_key_expiry() -> u64 { 365 }
fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

fn default_log_level() -> String { "info".to_string() }
fn default_log_file() -> String { "logs/redisgate.log".to_string() }

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        info!("Loading configuration from: {}", path.display());

        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        config.validate()?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Load configuration with environment overrides
    pub fn from_file_with_env<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut config = Self::from_file(path)?;
        config.apply_env_overrides()?;
        Ok(config)
    }

    /// Load from environment or default file
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = std::env::var("CONFIG_PATH")
            .unwrap_or_else(|_| {
                let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
                match env.as_str() {
                    "production" => "config.production.toml",
                    "test" => "config.test.toml",
                    _ => "config.toml",
                }.to_string()
            });

        info!("Loading config from: {}", config_path);
        Self::from_file_with_env(config_path)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Server overrides
        if let Ok(host) = std::env::var("SERVER_HOST") {
            info!("Override: SERVER_HOST = {}", host);
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            let port: u16 = port.parse().map_err(|_| {
                ConfigError::Validation("SERVER_PORT must be a valid port number".to_string())
            })?;
            info!("Override: SERVER_PORT = {}", port);
            self.server.port = port;
        }

        // Database overrides
        if let Ok(url) = std::env::var("DATABASE_URL") {
            info!("Override: DATABASE_URL (hidden)");
            self.database.url = url;
        }

        // Security overrides
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            info!("Override: JWT_SECRET (hidden)");
            self.security.jwt_secret = secret;
        }

        // Rate limit overrides
        if let Ok(rps) = std::env::var("RATE_LIMIT_RPS") {
            let rps: u32 = rps.parse().map_err(|_| {
                ConfigError::Validation("RATE_LIMIT_RPS must be a valid number".to_string())
            })?;
            info!("Override: RATE_LIMIT_RPS = {}", rps);
            self.rate_limit.default_requests_per_second = rps;
        }

        Ok(())
    }

    /// Validate configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate server config
        if self.server.port == 0 {
            return Err(ConfigError::Validation("Server port cannot be 0".to_string()));
        }
        if self.server.workers == 0 {
            return Err(ConfigError::Validation("Worker count must be at least 1".to_string()));
        }

        // Validate database config
        if self.database.url.is_empty() {
            return Err(ConfigError::Validation("Database URL is required".to_string()));
        }
        if self.database.max_connections == 0 {
            return Err(ConfigError::Validation("Max connections must be > 0".to_string()));
        }
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::Validation(
                "Min connections cannot exceed max connections".to_string()
            ));
        }

        // Validate security config
        if self.security.jwt_secret.is_empty() {
            return Err(ConfigError::Validation("JWT secret is required".to_string()));
        }
        if self.security.jwt_secret.len() < 32 {
            warn!("JWT secret is shorter than recommended 32 characters");
        }

        // Validate rate limit
        if self.rate_limit.enabled && self.rate_limit.default_requests_per_second == 0 {
            return Err(ConfigError::Validation(
                "Rate limit RPS must be > 0 when enabled".to_string()
            ));
        }

        info!("Configuration validation passed ✓");
        Ok(())
    }

    /// Get database URL (for migrations, etc.)
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Get server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

