
//! Message domain model
//!
//! Represents a QR code message shared in a channel.

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Maximum message size in bytes (5KB)
pub const MAX_MESSAGE_SIZE: usize = 5120;

/// Default message TTL in seconds (1 hour)
pub const DEFAULT_MESSAGE_TTL_SECS: u64 = 3600;

/// Message entity stored in memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: CompactString,
    pub name: CompactString,
    pub link: Arc<str>,
    pub link_domain: CompactString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<CompactString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<CompactString>,
    pub expire_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    /// Create a new message with the given parameters
    pub fn new(
        id: impl Into<CompactString>,
        name: impl Into<CompactString>,
        link: impl Into<Arc<str>>,
        ttl: Duration,
    ) -> Self {
        let link = link.into();
        let link_domain = Self::extract_domain(&link);
        let now = Utc::now();
        let ttl_duration = chrono::Duration::from_std(ttl)
            .unwrap_or_else(|_| chrono::Duration::seconds(DEFAULT_MESSAGE_TTL_SECS as i64));

        Self {
            id: id.into(),
            name: name.into(),
            link,
            link_domain,
            message_type: None,
            location: None,
            expire_at: now + ttl_duration,
            created_at: now,
        }
    }

    /// Check if this message has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expire_at
    }

    /// Extract the domain from a URL
    pub fn extract_domain(url: &str) -> CompactString {
        let url = url.trim();

        // Remove protocol
        let domain = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        // Extract domain (stop at first / or ?)
        let end = domain
            .find('/')
            .or_else(|| domain.find('?'))
            .unwrap_or(domain.len());

        // Remove port if present
        let domain = &domain[..end];
        let domain = domain.split(':').next().unwrap_or(domain);

        CompactString::from(domain)
    }

    /// Validate a link URL
    pub fn validate_link(link: &str) -> Result<(), String> {
        let link = link.trim();

        if link.is_empty() {
            return Err("Link is required".to_string());
        }

        if !link.starts_with("http://") && !link.starts_with("https://") {
            return Err("Link must start with http:// or https://".to_string());
        }

        Ok(())
    }

    /// Validate message size
    pub fn validate_size(
        name: &str,
        link: &str,
        message_type: Option<&str>,
        location: Option<&str>,
    ) -> Result<(), usize> {
        let total_size = name.len()
            + link.len()
            + message_type.map(|s| s.len()).unwrap_or(0)
            + location.map(|s| s.len()).unwrap_or(0);

        if total_size > MAX_MESSAGE_SIZE {
            return Err(total_size);
        }

        Ok(())
    }
}

/// Request to create a new message
#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub name: String,
    pub link: String,
    #[serde(default)]
    pub message_type: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default = "default_expire_seconds")]
    pub expire_seconds: u64,
}

fn default_expire_seconds() -> u64 {
    DEFAULT_MESSAGE_TTL_SECS
}

impl CreateMessageRequest {
    /// Validate the request
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Message name is required".to_string());
        }
        if self.name.len() > 100 {
            return Err("Message name must be at most 100 characters".to_string());
        }

        // Validate link
        Message::validate_link(&self.link)?;

        // Validate size
        if let Err(size) = Message::validate_size(
            &self.name,
            &self.link,
            self.message_type.as_deref(),
            self.location.as_deref(),
        ) {
            return Err(format!(
                "Message too large: {} bytes (max {})",
                size, MAX_MESSAGE_SIZE
            ));
        }

        // Validate expire_seconds
        if self.expire_seconds == 0 || self.expire_seconds > DEFAULT_MESSAGE_TTL_SECS {
            return Err(format!(
                "expire_seconds must be between 1 and {}",
                DEFAULT_MESSAGE_TTL_SECS
            ));
        }

        Ok(())
    }

    /// Get the link domain
    pub fn link_domain(&self) -> CompactString {
        Message::extract_domain(&self.link)
    }
}

/// Message response for API
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub name: String,
    pub link: String,
    pub link_domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub expire_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id.to_string(),
            name: message.name.to_string(),
            link: message.link.to_string(),
            link_domain: message.link_domain.to_string(),
            message_type: message.message_type.map(|s| s.to_string()),
            location: message.location.map(|s| s.to_string()),
            expire_at: message.expire_at,
            created_at: message.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new() {
        let message = Message::new(
            "msg123",
            "Test Message",
            "https://example.com/path",
            Duration::from_secs(3600),
        );

        assert_eq!(message.id, "msg123");
        assert_eq!(message.name, "Test Message");
        assert_eq!(message.link_domain, "example.com");
        assert!(!message.is_expired());
    }

    #[test]
    fn test_message_extract_domain() {
        assert_eq!(
            Message::extract_domain("https://example.com/path"),
            "example.com"
        );
        assert_eq!(
            Message::extract_domain("http://sub.domain.com:8080/path"),
            "sub.domain.com"
        );
        assert_eq!(
            Message::extract_domain("https://example.com?query=1"),
            "example.com"
        );
        assert_eq!(
            Message::extract_domain("https://example.com:443/path"),
            "example.com"
        );
    }

    #[test]
    fn test_message_validate_link() {
        assert!(Message::validate_link("https://example.com").is_ok());
        assert!(Message::validate_link("http://example.com").is_ok());
        assert!(Message::validate_link("ftp://example.com").is_err());
        assert!(Message::validate_link("example.com").is_err());
        assert!(Message::validate_link("").is_err());
    }

    #[test]
    fn test_message_validate_size() {
        assert!(Message::validate_size("name", "https://example.com", None, None).is_ok());

        let large_name = "x".repeat(6000);
        assert!(Message::validate_size(&large_name, "https://example.com", None, None).is_err());
    }

    #[test]
    fn test_create_message_request_validation() {
        let valid = CreateMessageRequest {
            name: "Test".to_string(),
            link: "https://example.com".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(valid.validate().is_ok());

        let empty_name = CreateMessageRequest {
            name: "".to_string(),
            link: "https://example.com".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(empty_name.validate().is_err());

        let invalid_link = CreateMessageRequest {
            name: "Test".to_string(),
            link: "not-a-url".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 3600,
        };
        assert!(invalid_link.validate().is_err());

        let invalid_expire = CreateMessageRequest {
            name: "Test".to_string(),
            link: "https://example.com".to_string(),
            message_type: None,
            location: None,
            expire_seconds: 0,
        };
        assert!(invalid_expire.validate().is_err());
    }

    #[test]
    fn test_message_is_expired() {
        let mut message = Message::new(
            "msg123",
            "Test",
            "https://example.com",
            Duration::from_secs(1),
        );

        // Not expired initially
        assert!(!message.is_expired());

        // Set expire_at to past
        message.expire_at = Utc::now() - chrono::Duration::seconds(10);
        assert!(message.is_expired());
    }
}
