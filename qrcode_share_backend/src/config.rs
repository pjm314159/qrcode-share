//! Application configuration
//!
//! This module defines the configuration structure and loading from environment variables.

use std::time::Duration;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    // Server configuration
    pub host: String,
    pub port: u16,

    // Database configuration
    pub database_url: String,
    pub db_max_connections: u32,
    pub db_min_connections: u32,

    // Channel limits
    pub max_channels: usize,
    pub max_messages_per_channel: usize,
    pub max_message_size: usize,
    pub message_ttl: Duration,
    pub channel_ttl: Duration,

    // Connection limits
    pub max_connections: usize,
    pub max_connections_per_channel: usize,

    // Rate limiting
    pub max_messages_per_minute: usize,
    pub max_channels_per_user: usize,

    // Performance tuning
    pub cleanup_interval: Duration,
    pub heartbeat_interval: Duration,
    pub connection_timeout: Duration,
    pub broadcast_buffer_size: usize,

    // WeChat configuration
    pub wx_appid: Option<String>,
    pub wx_secret: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://qrcode:qrcode_password@localhost:5432/qrcode_share".to_string()
            }),
            db_max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            db_min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            max_channels: std::env::var("MAX_CHANNELS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),
            max_messages_per_channel: std::env::var("MAX_MESSAGES_PER_CHANNEL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            max_message_size: std::env::var("MAX_MESSAGE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5120),
            message_ttl: Duration::from_secs(
                std::env::var("MESSAGE_TTL_SECONDS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3600),
            ),
            channel_ttl: Duration::from_secs(
                std::env::var("CHANNEL_TTL_DAYS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30)
                    * 24
                    * 3600,
            ),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            max_connections_per_channel: std::env::var("MAX_CONNECTIONS_PER_CHANNEL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            max_messages_per_minute: std::env::var("MAX_MESSAGES_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            max_channels_per_user: std::env::var("MAX_CHANNELS_PER_USER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            cleanup_interval: Duration::from_secs(
                std::env::var("CLEANUP_INTERVAL_SECONDS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(120),
            ),
            heartbeat_interval: Duration::from_secs(
                std::env::var("HEARTBEAT_INTERVAL_SECONDS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
            ),
            connection_timeout: Duration::from_secs(
                std::env::var("CONNECTION_TIMEOUT_SECONDS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
            ),
            broadcast_buffer_size: std::env::var("BROADCAST_BUFFER_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(256),
            wx_appid: std::env::var("WX_APPID").ok().and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            }),
            wx_secret: std::env::var("WX_SECRET").ok().and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            }),
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.port > 0, "PORT must be positive");
        anyhow::ensure!(!self.database_url.is_empty(), "DATABASE_URL is required");
        anyhow::ensure!(self.max_channels > 0, "MAX_CHANNELS must be positive");
        anyhow::ensure!(
            self.max_messages_per_channel > 0,
            "MAX_MESSAGES_PER_CHANNEL must be positive"
        );
        anyhow::ensure!(
            self.max_message_size > 0,
            "MAX_MESSAGE_SIZE must be positive"
        );
        anyhow::ensure!(
            self.db_max_connections > 0,
            "DB_MAX_CONNECTIONS must be positive"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::from_env().unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.max_channels, 5000);
        assert_eq!(config.max_messages_per_channel, 300);
        assert_eq!(config.max_message_size, 5120);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::from_env().unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_wechat_config_empty_string_is_none() {
        temp_env::with_vars(
            vec![
                ("WX_APPID", Some("")),
                ("WX_SECRET", Some("")),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.wx_appid, None, "Empty WX_APPID should be None");
                assert_eq!(config.wx_secret, None, "Empty WX_SECRET should be None");
            },
        );
    }

    #[test]
    fn test_wechat_config_valid_values() {
        temp_env::with_vars(
            vec![
                ("WX_APPID", Some("wx1234567890")),
                ("WX_SECRET", Some("secret123")),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.wx_appid, Some("wx1234567890".to_string()));
                assert_eq!(config.wx_secret, Some("secret123".to_string()));
            },
        );
    }

    #[test]
    fn test_wechat_config_missing_is_none() {
        temp_env::with_vars(
            vec![
                ("WX_APPID", Option::<&str>::None),
                ("WX_SECRET", Option::<&str>::None),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.wx_appid, None);
                assert_eq!(config.wx_secret, None);
            },
        );
    }

    #[test]
    fn test_wechat_config_partial_is_none() {
        temp_env::with_vars(
            vec![
                ("WX_APPID", Some("wx1234567890")),
                ("WX_SECRET", Some("")),
            ],
            || {
                let config = Config::from_env().unwrap();
                assert_eq!(config.wx_appid, Some("wx1234567890".to_string()));
                assert_eq!(config.wx_secret, None, "Empty WX_SECRET should be None");
            },
        );
    }
}
