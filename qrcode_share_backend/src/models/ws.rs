//! WebSocket message types
//!
//! Defines all message types for WebSocket communication.

use serde::{Deserialize, Serialize};

/// WebSocket message sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Connection established
    Connected {
        channel_id: String,
        subscriber_count: usize,
    },
    /// New message broadcast
    Message {
        id: String,
        name: String,
        link: String,
        message_type: Option<String>,
        location: Option<String>,
        created_at: i64,
        expire_at: i64,
    },
    /// Subscriber count updated
    SubscriberUpdate { count: usize },
    /// Error occurred
    Error { code: String, message: String },
    /// Pong response
    Pong,
}

/// WebSocket message sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Ping to keep connection alive
    Ping,
}

impl WsServerMessage {
    /// Create a connected message
    pub fn connected(channel_id: impl Into<String>, subscriber_count: usize) -> Self {
        Self::Connected {
            channel_id: channel_id.into(),
            subscriber_count,
        }
    }

    /// Create a message broadcast
    pub fn message(
        id: impl Into<String>,
        name: impl Into<String>,
        link: impl Into<String>,
        message_type: Option<String>,
        location: Option<String>,
        created_at: i64,
        expire_at: i64,
    ) -> Self {
        Self::Message {
            id: id.into(),
            name: name.into(),
            link: link.into(),
            message_type,
            location,
            created_at,
            expire_at,
        }
    }

    /// Create a subscriber update message
    pub fn subscriber_update(count: usize) -> Self {
        Self::SubscriberUpdate { count }
    }

    /// Create an error message
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create a pong message
    pub fn pong() -> Self {
        Self::Pong
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl WsClientMessage {
    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_message() {
        let msg = WsServerMessage::connected("abc123", 5);
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"type\":\"connected\""));
        assert!(json.contains("\"channel_id\":\"abc123\""));
        assert!(json.contains("\"subscriber_count\":5"));
    }

    #[test]
    fn test_message_broadcast() {
        let msg = WsServerMessage::message(
            "msg1",
            "John",
            "https://example.com",
            Some("lecture".to_string()),
            Some("Room 101".to_string()),
            1234567890,
            1234567890 + 3600,
        );
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("\"id\":\"msg1\""));
        assert!(json.contains("\"name\":\"John\""));
        assert!(json.contains("\"link\":\"https://example.com\""));
    }

    #[test]
    fn test_subscriber_update() {
        let msg = WsServerMessage::subscriber_update(10);
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"type\":\"subscriber_update\""));
        assert!(json.contains("\"count\":10"));
    }

    #[test]
    fn test_error_message() {
        let msg = WsServerMessage::error("CHANNEL_NOT_FOUND", "Channel does not exist");
        let json = msg.to_json().unwrap();

        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"code\":\"CHANNEL_NOT_FOUND\""));
        assert!(json.contains("\"message\":\"Channel does not exist\""));
    }

    #[test]
    fn test_pong_message() {
        let msg = WsServerMessage::pong();
        let json = msg.to_json().unwrap();

        assert_eq!(json, "{\"type\":\"pong\"}");
    }

    #[test]
    fn test_client_ping() {
        let json = "{\"type\":\"ping\"}";
        let msg = WsClientMessage::from_json(json).unwrap();

        matches!(msg, WsClientMessage::Ping);
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"type":"connected","channel_id":"abc","subscriber_count":3}"#;
        let msg: WsServerMessage = serde_json::from_str(json).unwrap();

        match msg {
            WsServerMessage::Connected {
                channel_id,
                subscriber_count,
            } => {
                assert_eq!(channel_id, "abc");
                assert_eq!(subscriber_count, 3);
            }
            _ => panic!("Expected Connected message"),
        }
    }
}
