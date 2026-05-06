//! Channel repository for database operations
//!
//! Provides CRUD operations for channels in PostgreSQL.

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::{Channel, CreateChannelRequest, UpdateChannelRequest};

/// Channel database record
#[derive(Debug, sqlx::FromRow)]
pub struct ChannelRow {
    pub id: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub link_limitation: serde_json::Value,
    pub channel_type: Option<String>,
    pub location: Option<String>,
    pub teacher: Option<String>,
    pub creator_ip: String,
    pub message_count: i32,
    pub last_activity: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<ChannelRow> for Channel {
    fn from(row: ChannelRow) -> Self {
        let link_limitation: Vec<String> =
            serde_json::from_value(row.link_limitation).unwrap_or_default();

        Self {
            id: row.id.into(),
            name: row.name.into(),
            has_password: row.password_hash.is_some(),
            link_limitation: if link_limitation.is_empty() {
                None
            } else {
                Some(link_limitation)
            },
            channel_type: row.channel_type.map(|s| s.into()),
            location: row.location.map(|s| s.into()),
            teacher: row.teacher.map(|s| s.into()),
            created_at: row.created_at,
            subscriber_count: 0,
            message_count: row.message_count as usize,
        }
    }
}

/// Channel repository for database operations
pub struct ChannelRepository {
    pool: PgPool,
}

impl ChannelRepository {
    /// Create a new channel repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new channel
    pub async fn create(
        &self,
        id: &str,
        request: &CreateChannelRequest,
        password_hash: Option<&str>,
        creator_ip: &str,
    ) -> Result<Channel, AppError> {
        let link_limitation = request
            .link_limitation
            .as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or(json!([])))
            .unwrap_or(json!([]));

        let row: ChannelRow = sqlx::query_as(
            r#"
            INSERT INTO channels (id, name, password_hash, link_limitation, channel_type, location, teacher, creator_ip)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(password_hash)
        .bind(&link_limitation)
        .bind(&request.channel_type)
        .bind(&request.location)
        .bind(&request.teacher)
        .bind(creator_ip)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create channel: {}", e)))?;

        Ok(row.into())
    }

    /// Find a channel by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Channel>, AppError> {
        let result: Result<Option<ChannelRow>, sqlx::Error> = sqlx::query_as(
            r#"
            SELECT id, name, password_hash, link_limitation, channel_type, location, teacher,
                   creator_ip, message_count, last_activity, created_at
            FROM channels
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(row.into())),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::DatabaseError(format!(
                "Failed to find channel: {}",
                e
            ))),
        }
    }

    /// Update a channel
    pub async fn update(
        &self,
        id: &str,
        request: &UpdateChannelRequest,
        password_hash: Option<&str>,
    ) -> Result<Channel, AppError> {
        let link_limitation = request
            .link_limitation
            .as_ref()
            .map(|v| serde_json::to_value(v).unwrap_or(json!([])));

        let row: ChannelRow = sqlx::query_as(
            r#"
            UPDATE channels
            SET name = COALESCE($2, name),
                password_hash = COALESCE($3, password_hash),
                link_limitation = COALESCE($4, link_limitation),
                channel_type = COALESCE($5, channel_type),
                location = COALESCE($6, location),
                teacher = COALESCE($7, teacher)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(password_hash)
        .bind(link_limitation.as_ref())
        .bind(&request.channel_type)
        .bind(&request.location)
        .bind(&request.teacher)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update channel: {}", e)))?;

        Ok(row.into())
    }

    /// Delete a channel
    pub async fn delete(&self, id: &str) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM channels WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete channel: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// List channels with pagination
    pub async fn list(
        &self,
        page: i64,
        limit: i64,
        channel_type: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<Channel>, AppError> {
        let offset = (page.saturating_sub(1)) * limit;

        let rows: Vec<ChannelRow> = sqlx::query_as(
            r#"
            SELECT id, name, password_hash, link_limitation, channel_type, location, teacher,
                   creator_ip, message_count, last_activity, created_at
            FROM channels
            WHERE ($1::varchar IS NULL OR channel_type = $1)
              AND ($2::varchar IS NULL OR name ILIKE '%' || $2 || '%')
            ORDER BY last_activity DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(channel_type)
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to list channels: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Count total channels
    pub async fn count(&self) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channels")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count channels: {}", e)))?;

        Ok(count.0)
    }

    /// Count channels by creator IP
    pub async fn count_by_ip(&self, ip: &str) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channels WHERE creator_ip = $1")
            .bind(ip)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to count channels by IP: {}", e))
            })?;

        Ok(count.0)
    }

    /// Increment message count
    pub async fn increment_message_count(&self, id: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE channels
            SET message_count = message_count + 1,
                last_activity = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to increment message count: {}", e))
        })?;

        Ok(())
    }

    /// Decrement message count
    pub async fn decrement_message_count(&self, id: &str, count: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE channels
            SET message_count = GREATEST(message_count - $2, 0)
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(count)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to decrement message count: {}", e))
        })?;

        Ok(())
    }

    /// Update last activity timestamp
    pub async fn update_last_activity(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE channels SET last_activity = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to update last activity: {}", e))
            })?;

        Ok(())
    }

    /// Clean up inactive channels (older than 30 days)
    pub async fn cleanup_inactive(&self) -> Result<u64, AppError> {
        let result =
            sqlx::query("DELETE FROM channels WHERE last_activity < NOW() - INTERVAL '30 days'")
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to cleanup channels: {}", e))
                })?;

        Ok(result.rows_affected())
    }

    /// Get password hash for a channel
    pub async fn get_password_hash(&self, id: &str) -> Result<Option<String>, AppError> {
        let result: Result<Option<(Option<String>,)>, sqlx::Error> =
            sqlx::query_as("SELECT password_hash FROM channels WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await;

        match result {
            Ok(Some((hash,))) => Ok(hash),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::DatabaseError(format!(
                "Failed to get password hash: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_row_conversion() {
        let row = ChannelRow {
            id: "test123".to_string(),
            name: "Test Channel".to_string(),
            password_hash: Some("hash".to_string()),
            link_limitation: json!(["example.com"]),
            channel_type: Some("sign-in".to_string()),
            location: Some("Room 101".to_string()),
            teacher: Some("Prof. Smith".to_string()),
            creator_ip: "127.0.0.1".to_string(),
            message_count: 5,
            last_activity: Utc::now(),
            created_at: Utc::now(),
        };

        let channel: Channel = row.into();
        assert_eq!(channel.id, "test123");
        assert_eq!(channel.name, "Test Channel");
        assert!(channel.has_password);
        assert_eq!(
            channel.link_limitation,
            Some(vec!["example.com".to_string()])
        );
    }
}
