//! Channel state management
//!
//! Manages in-memory channel data including messages and broadcast capabilities.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use compact_str::CompactString;
use dashmap::DashMap;
use tokio::sync::broadcast;

use crate::models::Message;

const MAX_MESSAGES_PER_CHANNEL: usize = 300;
const BROADCAST_BUFFER_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub enum ChannelEvent {
    Message(Arc<Message>),
    PreSerializedMessage(Arc<Vec<u8>>),
    SubscriberUpdate { count: usize },
}

/// In-memory channel state
#[derive(Debug)]
pub struct ChannelState {
    /// Channel ID
    pub id: CompactString,
    /// Channel name
    pub name: CompactString,
    /// Whether channel has password
    pub has_password: bool,
    /// Password hash
    pub password_hash: Option<String>,
    /// Allowed link domains
    pub link_limitation: Option<Vec<String>>,
    /// Channel type
    pub channel_type: Option<CompactString>,
    /// Location
    pub location: Option<CompactString>,
    /// Teacher name
    pub teacher: Option<CompactString>,
    /// Creator IP address
    pub creator_ip: Option<CompactString>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Messages stored in memory
    pub messages: Arc<DashMap<CompactString, Arc<Message>>>,
    /// Broadcast sender for real-time message delivery
    pub broadcast_tx: broadcast::Sender<ChannelEvent>,
    /// Number of active subscribers
    pub subscriber_count: AtomicUsize,
}

impl ChannelState {
    /// Create a new channel state
    pub fn new(id: impl Into<CompactString>, name: impl Into<CompactString>) -> Self {
        let (broadcast_tx, _) = broadcast::channel::<ChannelEvent>(BROADCAST_BUFFER_SIZE);

        Self {
            id: id.into(),
            name: name.into(),
            has_password: false,
            password_hash: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
            creator_ip: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            messages: Arc::new(DashMap::new()),
            broadcast_tx,
            subscriber_count: AtomicUsize::new(0),
        }
    }

    /// Create channel state with password
    pub fn with_password(mut self, password_hash: Option<String>) -> Self {
        self.has_password = password_hash.is_some();
        self.password_hash = password_hash;
        self
    }

    /// Set link limitation
    pub fn with_link_limitation(mut self, limitation: Vec<String>) -> Self {
        self.link_limitation = Some(limitation);
        self
    }

    /// Check if a link domain is allowed
    pub fn is_link_allowed(&self, domain: &str) -> bool {
        match &self.link_limitation {
            Some(allowed) => allowed.iter().any(|d| d == domain),
            None => true,
        }
    }

    /// Add a message to the channel
    pub fn add_message(&self, message: Arc<Message>) {
        if self.messages.len() >= MAX_MESSAGES_PER_CHANNEL {
            let mut oldest: Option<(CompactString, chrono::DateTime<Utc>)> = None;
            for entry in self.messages.iter() {
                let msg = entry.value();
                match &oldest {
                    None => oldest = Some((entry.key().clone(), msg.created_at)),
                    Some((_, oldest_time)) if msg.created_at < *oldest_time => {
                        oldest = Some((entry.key().clone(), msg.created_at));
                    }
                    _ => {}
                }
            }
            if let Some((id, _)) = oldest {
                self.messages.remove(&id);
            }
        }

        self.messages.insert(message.id.clone(), message.clone());

        let receiver_count = self.broadcast_tx.receiver_count();
        if receiver_count > 0 {
            let pre_serialized = crate::models::WsServerMessage::message(
                message.id.to_string(),
                message.name.to_string(),
                message.link.to_string(),
                message.message_type.as_ref().map(|s| s.to_string()),
                message.location.as_ref().map(|s| s.to_string()),
                message.created_at.timestamp(),
                message.expire_at.timestamp(),
            );
            if let Ok(json_bytes) = serde_json::to_vec(&pre_serialized) {
                let event = ChannelEvent::PreSerializedMessage(Arc::new(json_bytes));
                let result = self.broadcast_tx.send(event);
                tracing::info!(
                    "Broadcast pre-serialized message to channel {}: receivers={}, result={:?}",
                    self.id,
                    receiver_count,
                    result
                );
            } else {
                let result = self.broadcast_tx.send(ChannelEvent::Message(message));
                tracing::warn!(
                    "Fallback to Arc<Message> broadcast for channel {}: result={:?}",
                    self.id,
                    result
                );
            }
        }
    }

    /// Get a message by ID
    pub fn get_message(&self, id: &str) -> Option<Arc<Message>> {
        self.messages.get(id).map(|entry| entry.value().clone())
    }

    /// Get all non-expired messages
    pub fn get_messages(&self) -> Vec<Arc<Message>> {
        let now = Utc::now();
        self.messages
            .iter()
            .filter(|entry| entry.value().expire_at > now)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove expired messages
    pub fn remove_expired(&self) -> usize {
        let now = Utc::now();
        let mut removed = 0;

        self.messages.retain(|_, msg| {
            let should_keep = msg.expire_at > now;
            if !should_keep {
                removed += 1;
            }
            should_keep
        });

        removed
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscriber_count.load(Ordering::Relaxed)
    }

    /// Increment subscriber count
    pub fn inc_subscribers(&self) {
        self.subscriber_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement subscriber count
    pub fn dec_subscribers(&self) {
        self.subscriber_count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Subscribe to broadcast messages
    pub fn subscribe(&self) -> broadcast::Receiver<ChannelEvent> {
        self.broadcast_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as StdDuration;

    fn create_test_message(id: &str, name: &str) -> Arc<Message> {
        Arc::new(Message::new(
            id,
            name,
            "https://example.com",
            StdDuration::from_secs(3600),
        ))
    }

    #[test]
    fn test_channel_state_new() {
        let channel = ChannelState::new("test123", "Test Channel");

        assert_eq!(channel.id, "test123");
        assert_eq!(channel.name, "Test Channel");
        assert!(!channel.has_password);
        assert_eq!(channel.message_count(), 0);
        assert_eq!(channel.subscriber_count(), 0);
    }

    #[test]
    fn test_channel_state_with_options() {
        let channel = ChannelState::new("test123", "Test")
            .with_password(Some("$2b$12$test_hash".to_string()))
            .with_link_limitation(vec!["safe.com".to_string()]);

        assert!(channel.has_password);
        assert!(channel.is_link_allowed("safe.com"));
        assert!(!channel.is_link_allowed("evil.com"));
    }

    #[test]
    fn test_channel_add_message() {
        let channel = ChannelState::new("test", "Test");
        let msg = create_test_message("msg1", "Test Message");

        channel.add_message(msg);

        assert_eq!(channel.message_count(), 1);
        assert!(channel.get_message("msg1").is_some());
    }

    #[test]
    fn test_channel_eviction() {
        let channel = ChannelState::new("test", "Test");

        // Add more than max messages
        for i in 0..(MAX_MESSAGES_PER_CHANNEL + 10) {
            let msg = create_test_message(&format!("msg{}", i), &format!("Message {}", i));
            channel.add_message(msg);
        }

        // Should be at max capacity
        assert_eq!(channel.message_count(), MAX_MESSAGES_PER_CHANNEL);
    }

    #[test]
    fn test_channel_subscriber_count() {
        let channel = ChannelState::new("test", "Test");

        channel.inc_subscribers();
        channel.inc_subscribers();
        assert_eq!(channel.subscriber_count(), 2);

        channel.dec_subscribers();
        assert_eq!(channel.subscriber_count(), 1);
    }

    #[test]
    fn test_channel_subscribe() {
        let channel = ChannelState::new("test", "Test");
        let _rx = channel.subscribe();

        // Add a message
        let msg = create_test_message("msg1", "Test");
        channel.add_message(msg.clone());

        // Should receive the broadcast
        // Note: In async context, we'd use rx.recv().await
        // For sync test, we just verify the channel exists
        assert!(channel.broadcast_tx.receiver_count() >= 1);
    }
}
