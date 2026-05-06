//! Database connection management
//!
//! Provides connection pool and database initialization.

use std::time::Duration;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppError;

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    /// Connection pool
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let connect_options = PgConnectOptions::new()
            .host(
                config
                    .database_url
                    .split('@')
                    .nth(1)
                    .unwrap_or("localhost")
                    .split(':')
                    .next()
                    .unwrap_or("localhost"),
            )
            .port(5432)
            .username(
                config
                    .database_url
                    .split('@')
                    .nth(0)
                    .unwrap_or("")
                    .split(':')
                    .nth(1)
                    .unwrap_or("qrcode"),
            )
            .password(
                config
                    .database_url
                    .split(':')
                    .nth(2)
                    .unwrap_or("")
                    .split('@')
                    .nth(0)
                    .unwrap_or(""),
            )
            .database(
                config
                    .database_url
                    .split('/')
                    .nth(3)
                    .unwrap_or("qrcode_share"),
            );

        let pool = PgPoolOptions::new()
            .max_connections(config.db_max_connections)
            .min_connections(config.db_min_connections)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect_with(connect_options)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to connect to database: {}", e))
            })?;

        Ok(Self { pool })
    }

    /// Create from a database URL string (for testing)
    pub async fn new_from_url(database_url: &str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await
            .expect("Failed to create database pool");

        Self { pool }
    }

    /// Create from existing pool (for testing)
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<(), AppError> {
        sqlx::migrate!("../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Health check - verify database is accessible
    pub async fn health_check(&self) -> Result<bool, AppError> {
        let result: Result<(i64,), sqlx::Error> =
            sqlx::query_as("SELECT 1").fetch_one(&self.pool).await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(AppError::DatabaseError(format!(
                "Health check failed: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_clone() {
        // Test that Database is cloneable
        fn assert_clone<T: Clone>() {}
        assert_clone::<Database>();
    }
}
