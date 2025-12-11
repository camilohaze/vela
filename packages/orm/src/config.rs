/*!
Database configuration for Vela ORM.

This module provides configuration structures for different database drivers
and connection pooling settings.
*/

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Supported database drivers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseDriver {
    /// PostgreSQL database
    Postgres,
    /// MySQL database
    Mysql,
    /// SQLite database
    Sqlite,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database driver to use
    pub driver: DatabaseDriver,

    /// Database host (not used for SQLite)
    #[serde(default)]
    pub host: Option<String>,

    /// Database port (not used for SQLite)
    #[serde(default)]
    pub port: Option<u16>,

    /// Database name or SQLite file path
    pub database: String,

    /// Database username (not used for SQLite)
    #[serde(default)]
    pub username: Option<String>,

    /// Database password (not used for SQLite)
    #[serde(default)]
    pub password: Option<String>,

    /// Connection pool settings
    #[serde(default)]
    pub pool: ConnectionPoolConfig,

    /// SSL/TLS settings
    #[serde(default)]
    pub ssl: SslConfig,

    /// Additional driver-specific options
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Maximum time to wait for a connection
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: Duration,

    /// Maximum lifetime of a connection
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: Duration,

    /// Maximum idle time for a connection
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: Duration,

    /// Health check interval
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout: default_connection_timeout(),
            max_lifetime: default_max_lifetime(),
            idle_timeout: default_idle_timeout(),
            health_check_interval: default_health_check_interval(),
        }
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// Whether to use SSL/TLS
    #[serde(default)]
    pub enabled: bool,

    /// SSL mode (for PostgreSQL)
    #[serde(default)]
    pub mode: SslMode,

    /// Path to CA certificate file
    #[serde(default)]
    pub ca_cert: Option<String>,

    /// Path to client certificate file
    #[serde(default)]
    pub client_cert: Option<String>,

    /// Path to client key file
    #[serde(default)]
    pub client_key: Option<String>,

    /// Accept invalid certificates (for development)
    #[serde(default)]
    pub accept_invalid_certs: bool,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: SslMode::Prefer,
            ca_cert: None,
            client_cert: None,
            client_key: None,
            accept_invalid_certs: false,
        }
    }
}

/// SSL modes for PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    /// Only try an SSL connection
    Require,
    /// Prefer SSL, but allow non-SSL
    #[default]
    Prefer,
    /// Allow SSL, but prefer non-SSL
    Allow,
    /// Disable SSL
    Disable,
}

// Default values
fn default_max_connections() -> u32 { 20 }
fn default_min_connections() -> u32 { 5 }
fn default_connection_timeout() -> Duration { Duration::from_secs(30) }
fn default_max_lifetime() -> Duration { Duration::from_secs(30 * 60) } // 30 minutes
fn default_idle_timeout() -> Duration { Duration::from_secs(10 * 60) } // 10 minutes
fn default_health_check_interval() -> Duration { Duration::from_secs(30) }

impl DatabaseConfig {
    /// Create a PostgreSQL configuration
    pub fn postgres(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            driver: DatabaseDriver::Postgres,
            host: Some(host.to_string()),
            port: Some(port),
            database: database.to_string(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            pool: ConnectionPoolConfig::default(),
            ssl: SslConfig::default(),
            options: std::collections::HashMap::new(),
        }
    }

    /// Create a MySQL configuration
    pub fn mysql(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            driver: DatabaseDriver::Mysql,
            host: Some(host.to_string()),
            port: Some(port),
            database: database.to_string(),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            pool: ConnectionPoolConfig::default(),
            ssl: SslConfig::default(),
            options: std::collections::HashMap::new(),
        }
    }

    /// Create a SQLite configuration
    pub fn sqlite(path: &str) -> Self {
        Self {
            driver: DatabaseDriver::Sqlite,
            host: None,
            port: None,
            database: path.to_string(),
            username: None,
            password: None,
            pool: ConnectionPoolConfig {
                max_connections: 1, // SQLite doesn't benefit from connection pooling
                min_connections: 1,
                connection_timeout: Duration::from_secs(30),
                max_lifetime: Duration::from_secs(u64::MAX), // Never expire
                idle_timeout: Duration::from_secs(u64::MAX), // Never idle
                health_check_interval: Duration::from_secs(300),
            },
            ssl: SslConfig::default(),
            options: std::collections::HashMap::new(),
        }
    }

    /// Create configuration from connection URL
    pub fn from_url(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let url = url::Url::parse(url)?;

        let driver = match url.scheme() {
            "postgres" | "postgresql" => DatabaseDriver::Postgres,
            "mysql" => DatabaseDriver::Mysql,
            "sqlite" => DatabaseDriver::Sqlite,
            _ => return Err("Unsupported database driver".into()),
        };

        let host = url.host_str().map(|s| s.to_string());
        let port = url.port();
        let database = url.path().trim_start_matches('/').to_string();
        let username = if !url.username().is_empty() {
            Some(url.username().to_string())
        } else {
            None
        };
        let password = url.password().map(|s| s.to_string());

        Ok(Self {
            driver,
            host,
            port,
            database,
            username,
            password,
            pool: ConnectionPoolConfig::default(),
            ssl: SslConfig::default(),
            options: std::collections::HashMap::new(),
        })
    }

    /// Get connection URL
    pub fn to_url(&self) -> String {
        match &self.driver {
            DatabaseDriver::Postgres => {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.username.as_deref().unwrap_or(""),
                    self.password.as_deref().unwrap_or(""),
                    self.host.as_deref().unwrap_or("localhost"),
                    self.port.unwrap_or(5432),
                    self.database
                )
            }
            DatabaseDriver::Mysql => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username.as_deref().unwrap_or(""),
                    self.password.as_deref().unwrap_or(""),
                    self.host.as_deref().unwrap_or("localhost"),
                    self.port.unwrap_or(3306),
                    self.database
                )
            }
            DatabaseDriver::Sqlite => {
                format!("sqlite://{}", self.database)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_config() {
        let config = DatabaseConfig::postgres("localhost", 5432, "testdb", "user", "pass");
        assert!(matches!(config.driver, DatabaseDriver::Postgres));
        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
        assert_eq!(config.database, "testdb");
    }

    #[test]
    fn test_sqlite_config() {
        let config = DatabaseConfig::sqlite("/tmp/test.db");
        assert!(matches!(config.driver, DatabaseDriver::Sqlite));
        assert_eq!(config.database, "/tmp/test.db");
        assert_eq!(config.pool.max_connections, 1); // SQLite specific
    }

    #[test]
    fn test_config_to_url() {
        let config = DatabaseConfig::postgres("localhost", 5432, "testdb", "user", "pass");
        let url = config.to_url();
        assert_eq!(url, "postgres://user:pass@localhost:5432/testdb");
    }
}