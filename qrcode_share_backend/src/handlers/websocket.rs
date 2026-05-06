//! WebSocket handler for real-time message delivery
//!
//! Provides WebSocket endpoint for subscribing to channel messages.

use std::time::Duration;

use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::auth::check_channel_access;
use crate::error::AppError;
use crate::models::{WsClientMessage, WsServerMessage};
use crate::state::{AppState, ChannelEvent};

/// WebSocket connection timeout (60 seconds of inactivity)
const WEBSOCKET_TIMEOUT_SECS: u64 = 60;

/// Heartbeat interval (30 seconds)
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    /// Password for protected channels
    pub password: Option<String>,
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    State(app_state): State<AppState>,
    Path(channel_id): Path<String>,
    Query(query): Query<WebSocketQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    // Get channel
    let channel = app_state
        .get_channel(&channel_id)
        .ok_or_else(|| AppError::ChannelNotFound(channel_id.clone()))?;

    // Check password if required
    if channel.has_password {
        let access = check_channel_access(
            channel.has_password,
            query.password.as_deref(),
            channel.password_hash.as_deref(),
        )
        .map_err(AppError::Internal)?;

        if !access {
            return Err(AppError::PasswordRequired);
        }
    }

    // Upgrade to WebSocket
    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, app_state, channel_id)))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, app_state: AppState, channel_id: String) {
    let channel = match app_state.get_channel(&channel_id) {
        Some(c) => c,
        None => {
            warn!("Channel {} not found during WebSocket handling", channel_id);
            return;
        }
    };

    // Increment subscriber count
    channel.inc_subscribers();
    app_state.metrics.inc_subscribers();
    let subscriber_count = channel.subscriber_count();
    info!(
        "WebSocket connected to channel {}, subscribers: {}",
        channel_id, subscriber_count
    );

    // Broadcast subscriber update
    broadcast_subscriber_update(&channel_id, subscriber_count, &app_state);

    // Split socket into sender and receiver
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Send connected message
    let connected_msg = WsServerMessage::connected(&channel_id, subscriber_count);
    if let Ok(json) = connected_msg.to_json() {
        if ws_tx.send(WsMessage::Text(json)).await.is_err() {
            warn!("Failed to send connected message");
            channel.dec_subscribers();
            return;
        }
    }

    // Subscribe to channel broadcasts
    let mut broadcast_rx = channel.subscribe();
    let mut heartbeat_interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
    let mut timeout_counter = 0u64;

    // Main WebSocket loop
    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);
                        timeout_counter = 0; // Reset timeout on any message

                        // Parse client message
                        match WsClientMessage::from_json(&text) {
                            Ok(WsClientMessage::Ping) => {
                                let pong = WsServerMessage::pong();
                                if let Ok(json) = pong.to_json() {
                                    if ws_tx.send(WsMessage::Text(json)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse WebSocket message: {}", e);
                            }
                        }
                    }
                    Some(Ok(WsMessage::Ping(data))) => {
                        debug!("Received ping: {:?}", data);
                        timeout_counter = 0;
                        if ws_tx.send(WsMessage::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(WsMessage::Pong(_))) => {
                        debug!("Received pong");
                        timeout_counter = 0;
                    }
                    Some(Ok(WsMessage::Close(_))) => {
                        info!("WebSocket closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        break;
                    }
                    _ => {}
                }
            }

            // Handle broadcast events from channel
            event = broadcast_rx.recv() => {
                match event {
                    Ok(ChannelEvent::PreSerializedMessage(json_bytes)) => {
                        let text = String::from_utf8_lossy(&json_bytes).to_string();
                        if ws_tx.send(WsMessage::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Ok(ChannelEvent::Message(message)) => {
                        let broadcast_msg = WsServerMessage::message(
                            message.id.to_string(),
                            message.name.to_string(),
                            message.link.to_string(),
                            message.message_type.as_ref().map(|s| s.to_string()),
                            message.location.as_ref().map(|s| s.to_string()),
                            message.created_at.timestamp(),
                            message.expire_at.timestamp(),
                        );

                        if let Ok(json) = broadcast_msg.to_json() {
                            if ws_tx.send(WsMessage::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Ok(ChannelEvent::SubscriberUpdate { count }) => {
                        let update_msg = WsServerMessage::subscriber_update(count);
                        if let Ok(json) = update_msg.to_json() {
                            if ws_tx.send(WsMessage::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("Broadcast channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Broadcast channel lagged by {} messages", n);
                    }
                }
            }

            // Heartbeat tick
            _ = heartbeat_interval.tick() => {
                timeout_counter += HEARTBEAT_INTERVAL_SECS;

                // Check for timeout
                if timeout_counter >= WEBSOCKET_TIMEOUT_SECS {
                    info!("WebSocket timeout after {} seconds", timeout_counter);
                    break;
                }

                // Send ping to client
                if ws_tx.send(WsMessage::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup on disconnect
    channel.dec_subscribers();
    app_state.metrics.dec_subscribers();
    let new_count = channel.subscriber_count();
    info!(
        "WebSocket disconnected from channel {}, subscribers: {}",
        channel_id, new_count
    );

    // Broadcast subscriber update
    broadcast_subscriber_update(&channel_id, new_count, &app_state);
}

/// Broadcast subscriber count update to all clients
fn broadcast_subscriber_update(channel_id: &str, count: usize, app_state: &AppState) {
    if let Some(channel) = app_state.get_channel(channel_id) {
        let receiver_count = channel.broadcast_tx.receiver_count();
        let result = channel
            .broadcast_tx
            .send(ChannelEvent::SubscriberUpdate { count });
        tracing::info!(
            "Broadcasting subscriber update: {} -> {} (receivers={}, result={:?})",
            channel_id,
            count,
            receiver_count,
            result
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_query_deserialization() {
        let query: WebSocketQuery = serde_urlencoded::from_str("password=secret").unwrap();
        assert_eq!(query.password, Some("secret".to_string()));

        let query: WebSocketQuery = serde_urlencoded::from_str("").unwrap();
        assert!(query.password.is_none());
    }

    #[test]
    fn test_websocket_timeout_constant() {
        assert_eq!(WEBSOCKET_TIMEOUT_SECS, 60);
    }

    #[test]
    fn test_heartbeat_interval_constant() {
        assert_eq!(HEARTBEAT_INTERVAL_SECS, 30);
    }

    #[test]
    fn test_server_message_creation() {
        let connected = WsServerMessage::connected("abc123", 5);
        match connected {
            WsServerMessage::Connected {
                channel_id,
                subscriber_count,
            } => {
                assert_eq!(channel_id, "abc123");
                assert_eq!(subscriber_count, 5);
            }
            _ => panic!("Expected Connected message"),
        }

        let msg = WsServerMessage::message(
            "msg1",
            "Test",
            "https://example.com",
            None,
            None,
            1234567890,
            1234567890 + 3600,
        );
        match msg {
            WsServerMessage::Message { id, name, link, .. } => {
                assert_eq!(id, "msg1");
                assert_eq!(name, "Test");
                assert_eq!(link, "https://example.com");
            }
            _ => panic!("Expected Message variant"),
        }

        let update = WsServerMessage::subscriber_update(10);
        match update {
            WsServerMessage::SubscriberUpdate { count } => {
                assert_eq!(count, 10);
            }
            _ => panic!("Expected SubscriberUpdate message"),
        }

        let error = WsServerMessage::error("TEST_ERROR", "Test error message");
        match error {
            WsServerMessage::Error { code, message } => {
                assert_eq!(code, "TEST_ERROR");
                assert_eq!(message, "Test error message");
            }
            _ => panic!("Expected Error message"),
        }

        let pong = WsServerMessage::pong();
        match pong {
            WsServerMessage::Pong => {}
            _ => panic!("Expected Pong message"),
        }
    }

    #[test]
    fn test_client_message_parsing() {
        let ping = WsClientMessage::from_json("{\"type\":\"ping\"}").unwrap();
        matches!(ping, WsClientMessage::Ping);
    }
}
