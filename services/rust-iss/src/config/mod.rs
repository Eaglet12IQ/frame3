use serde::Deserialize;
use std::env;
use std::time::Duration;

/// Application configuration structure
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: Option<RedisConfig>,
    pub nasa: NasaConfig,
    pub iss: IssConfig,
    pub spacex: SpaceXConfig,
    pub http_client: HttpClientConfig,
    pub server: ServerConfig,
    pub osdr: OsdrConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct NasaConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub fetch_intervals: NasaFetchIntervals,
}

#[derive(Debug, Clone)]
pub struct NasaFetchIntervals {
    pub apod: u64,
    pub neo: u64,
    pub donki: u64,
}

#[derive(Debug, Clone)]
pub struct IssConfig {
    pub api_url: String,
    pub fetch_interval: u64,
}

#[derive(Debug, Clone)]
pub struct SpaceXConfig {
    pub fetch_interval: u64,
}

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub user_agent: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct OsdrConfig {
    pub api_url: String,
    pub fetch_interval: u64,
    pub list_limit: i64,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl AppConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env(),
            nasa: NasaConfig::from_env()?,
            iss: IssConfig::from_env()?,
            spacex: SpaceXConfig::from_env()?,
            http_client: HttpClientConfig::from_env()?,
            server: ServerConfig::from_env()?,
            osdr: OsdrConfig::from_env()?,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.database.validate()?;
        if let Some(ref redis) = &self.redis {
            redis.validate()?;
        }
        self.nasa.validate()?;
        self.iss.validate()?;
        self.spacex.validate()?;
        self.http_client.validate()?;
        self.server.validate()?;
        self.osdr.validate()?;
        Ok(())
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingRequired("DATABASE_URL".to_string()))?;

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .map_err(|_| ConfigError::InvalidValue("DATABASE_MAX_CONNECTIONS must be a valid u32".to_string()))?;

        Ok(Self { url, max_connections })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::InvalidValue("DATABASE_URL cannot be empty".to_string()));
        }
        if self.max_connections == 0 {
            return Err(ConfigError::InvalidValue("DATABASE_MAX_CONNECTIONS must be greater than 0".to_string()));
        }
        Ok(())
    }
}

impl RedisConfig {
    fn from_env() -> Option<Self> {
        env::var("REDIS_URL").ok().map(|url| Self { url })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.url.is_empty() {
            return Err(ConfigError::InvalidValue("REDIS_URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl NasaConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let api_url = env::var("NASA_API_URL")
            .unwrap_or_else(|_| "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string());

        let api_key = env::var("NASA_API_KEY").ok();

        let fetch_intervals = NasaFetchIntervals::from_env()?;

        Ok(Self { api_url, api_key, fetch_intervals })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.api_url.is_empty() {
            return Err(ConfigError::InvalidValue("NASA_API_URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl NasaFetchIntervals {
    fn from_env() -> Result<Self, ConfigError> {
        let apod = env_u64("APOD_EVERY_SECONDS", 43200)?;
        let neo = env_u64("NEO_EVERY_SECONDS", 7200)?;
        let donki = env_u64("DONKI_EVERY_SECONDS", 3600)?;

        Ok(Self { apod, neo, donki })
    }
}

impl IssConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let api_url = env::var("WHERE_ISS_URL")
            .unwrap_or_else(|_| "https://api.wheretheiss.at/v1/satellites/25544".to_string());

        let fetch_interval = env_u64("ISS_EVERY_SECONDS", 120)?;

        Ok(Self { api_url, fetch_interval })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.api_url.is_empty() {
            return Err(ConfigError::InvalidValue("WHERE_ISS_URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl SpaceXConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let fetch_interval = env_u64("SPACEX_EVERY_SECONDS", 3600)?;
        Ok(Self { fetch_interval })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl HttpClientConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let timeout_secs = env_u64("HTTP_TIMEOUT_SECONDS", 30)?;
        let max_retries = env_u64("HTTP_MAX_RETRIES", 3)? as u32;
        let retry_delay_ms = env_u64("HTTP_RETRY_DELAY_MS", 1000)?;

        Ok(Self {
            timeout: Duration::from_secs(timeout_secs),
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
            user_agent: env::var("HTTP_USER_AGENT")
                .unwrap_or_else(|_| "Rust-ISS-Service/1.0".to_string()),
        })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidValue("HTTP_TIMEOUT_SECONDS must be greater than 0".to_string()));
        }
        if self.max_retries == 0 {
            return Err(ConfigError::InvalidValue("HTTP_MAX_RETRIES must be greater than 0".to_string()));
        }
        Ok(())
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            user_agent: "Rust-ISS-Service/1.0".to_string(),
        }
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue("SERVER_PORT must be a valid u16".to_string()))?;

        Ok(Self { host, port })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.host.is_empty() {
            return Err(ConfigError::InvalidValue("SERVER_HOST cannot be empty".to_string()));
        }
        Ok(())
    }
}

impl OsdrConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let api_url = env::var("OSDR_API_URL")
            .unwrap_or_else(|_| "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string());
        let fetch_interval = env_u64("FETCH_EVERY_SECONDS", 600)?;
        let list_limit = env::var("OSDR_LIST_LIMIT")
            .unwrap_or_else(|_| "20".to_string())
            .parse::<i64>()
            .map_err(|_| ConfigError::InvalidValue("OSDR_LIST_LIMIT must be a valid i64".to_string()))?;

        Ok(Self { api_url, fetch_interval, list_limit })
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.api_url.is_empty() {
            return Err(ConfigError::InvalidValue("OSDR_API_URL cannot be empty".to_string()));
        }
        if self.list_limit <= 0 {
            return Err(ConfigError::InvalidValue("OSDR_LIST_LIMIT must be greater than 0".to_string()));
        }
        Ok(())
    }
}

/// Configuration error types
#[derive(Debug, Clone)]
pub enum ConfigError {
    MissingRequired(String),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingRequired(var) => write!(f, "Missing required environment variable: {}", var),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Helper function to parse environment variable as u64 with default
fn env_u64(key: &str, default: u64) -> Result<u64, ConfigError> {
    env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidValue(format!("{} must be a valid u64", key)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_u64_with_valid_value() {
        env::set_var("TEST_U64", "42");
        let result = env_u64("TEST_U64", 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        env::remove_var("TEST_U64");
    }

    #[test]
    fn test_env_u64_with_default() {
        env::remove_var("TEST_U64_DEFAULT");
        let result = env_u64("TEST_U64_DEFAULT", 99);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99);
    }

    #[test]
    fn test_env_u64_with_invalid_value() {
        env::set_var("TEST_U64_INVALID", "not_a_number");
        let result = env_u64("TEST_U64_INVALID", 10);
        assert!(result.is_err());
        env::remove_var("TEST_U64_INVALID");
    }

    #[test]
    fn test_database_config_from_env() {
        env::set_var("DATABASE_URL", "postgres://user:pass@localhost/db");
        env::set_var("DATABASE_MAX_CONNECTIONS", "10");
        let config = DatabaseConfig::from_env();
        assert!(config.is_ok());
        let db_config = config.unwrap();
        assert_eq!(db_config.url, "postgres://user:pass@localhost/db");
        assert_eq!(db_config.max_connections, 10);
        env::remove_var("DATABASE_URL");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }

    #[test]
    fn test_database_config_validate() {
        let valid_config = DatabaseConfig {
            url: "postgres://test".to_string(),
            max_connections: 5,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = DatabaseConfig {
            url: "".to_string(),
            max_connections: 5,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_server_config_from_env() {
        env::set_var("SERVER_HOST", "127.0.0.1");
        env::set_var("SERVER_PORT", "8080");
        let config = ServerConfig::from_env();
        assert!(config.is_ok());
        let server_config = config.unwrap();
        assert_eq!(server_config.host, "127.0.0.1");
        assert_eq!(server_config.port, 8080);
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_server_config_validate() {
        let valid_config = ServerConfig {
            host: "localhost".to_string(),
            port: 3000,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = ServerConfig {
            host: "".to_string(),
            port: 3000,
        };
        assert!(invalid_config.validate().is_err());
    }
}
