//! Channel domain model
//!
//! Represents a channel where QR code messages are shared.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// Channel entity stored in database and memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: CompactString,
    pub name: CompactString,
    pub has_password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_limitation: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<CompactString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<CompactString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher: Option<CompactString>,
    pub created_at: DateTime<Utc>,
    pub subscriber_count: usize,
    pub message_count: usize,
}

impl Channel {
    /// Create a new channel with the given ID and name
    pub fn new(id: impl Into<CompactString>, name: impl Into<CompactString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            has_password: false,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
            created_at: Utc::now(),
            subscriber_count: 0,
            message_count: 0,
        }
    }

    /// Check if a link domain is allowed for this channel
    pub fn is_link_allowed(&self, domain: &str) -> bool {
        match &self.link_limitation {
            Some(allowed) => allowed.iter().any(|d| d == domain),
            None => true,
        }
    }
}

/// Request to create a new channel
#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub link_limitation: Option<Vec<String>>,
    #[serde(default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub teacher: Option<String>,
}

impl CreateChannelRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Channel name is required".to_string());
        }
        if self.name.len() > 100 {
            return Err("Channel name must be at most 100 characters".to_string());
        }
        if let Some(ref password) = self.password {
            if password.len() > 64 {
                return Err("Password must be at most 64 characters".to_string());
            }
        }
        Ok(())
    }
}

/// Request to update an existing channel
#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub link_limitation: Option<Vec<String>>,
    #[serde(default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub teacher: Option<String>,
}

/// Channel response for API
#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: String,
    pub name: String,
    pub has_password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_limitation: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teacher: Option<String>,
    pub created_at: DateTime<Utc>,
    pub subscriber_count: usize,
    pub message_count: usize,
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id.to_string(),
            name: channel.name.to_string(),
            has_password: channel.has_password,
            link_limitation: channel.link_limitation,
            channel_type: channel.channel_type.map(|s| s.to_string()),
            location: channel.location.map(|s| s.to_string()),
            teacher: channel.teacher.map(|s| s.to_string()),
            created_at: channel.created_at,
            subscriber_count: channel.subscriber_count,
            message_count: channel.message_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_new() {
        let channel = Channel::new("abc123", "Test Channel");
        assert_eq!(channel.id, "abc123");
        assert_eq!(channel.name, "Test Channel");
        assert!(!channel.has_password);
        assert_eq!(channel.subscriber_count, 0);
        assert_eq!(channel.message_count, 0);
    }

    #[test]
    fn test_channel_is_link_allowed() {
        let mut channel = Channel::new("abc123", "Test");

        // No limitation - all domains allowed
        assert!(channel.is_link_allowed("example.com"));
        assert!(channel.is_link_allowed("evil.com"));

        // With limitation
        channel.link_limitation = Some(vec!["safe.com".to_string(), "trusted.com".to_string()]);
        assert!(channel.is_link_allowed("safe.com"));
        assert!(channel.is_link_allowed("trusted.com"));
        assert!(!channel.is_link_allowed("evil.com"));
    }

    #[test]
    fn test_create_channel_request_validation() {
        let valid = CreateChannelRequest {
            name: "Valid Channel".to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        assert!(valid.validate().is_ok());

        let empty_name = CreateChannelRequest {
            name: "".to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        assert!(empty_name.validate().is_err());

        let long_name = CreateChannelRequest {
            name: "x".repeat(101),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        };
        assert!(long_name.validate().is_err());
    }
}
