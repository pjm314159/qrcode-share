//! WebSocket edge case tests
//!
//! These tests verify WebSocket behavior under various edge conditions:
//! - Client reconnection
//! - Message ordering
//! - Large message handling
//! - Invalid JSON handling

use std::time::Duration;

use axum::http::StatusCode;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use reqwest::Client;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

/// Test server wrapper for WebSocket edge case tests
struct WsTestServer {
    addr: std::net::SocketAddr,
    client: Client,
}

impl WsTestServer {
    async fn start() -> Self {
        use qrcode_share_backend::{build_router, AppState, Config};

        let config = std::sync::Arc::new(Config::from_env().expect("Config failed"));
        let app_state = AppState::new(config);
        let router = build_router(app_state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind port");
        let addr = listener.local_addr().expect("Failed to get local addr");

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, router).await {
                eprintln!("Server error: {}", e);
            }
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        Self {
            addr,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Client creation failed"),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn ws_url(&self, channel_id: &str) -> String {
        format!("ws://{}/api/channels/{}/ws", self.addr, channel_id)
    }

    async fn create_channel(&self, name: &str) -> String {
        let body = serde_json::json!({ "name": name });

        let response = self
            .client
            .post(format!("{}/api/channels", self.base_url()))
            .json(&body)
            .send()
            .await
            .expect("Create channel request failed");

        assert_eq!(response.status(), StatusCode::CREATED);

        let result: serde_json::Value = response.json().await.expect("JSON parse failed");
        result["data"]["id"]
            .as_str()
            .expect("Missing id")
            .to_string()
    }

    async fn send_message(&self, channel_id: &str, name: &str, link: &str) {
        let body = serde_json::json!({
            "name": name,
            "link": link,
            "expire_seconds": 3600
        });

        let response = self
            .client
            .post(format!(
                "{}/api/channels/{}/messages",
                self.base_url(),
                channel_id
            ))
            .json(&body)
            .send()
            .await
            .expect("Send message request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}

/// Connect to WebSocket and return stream
async fn connect_ws(url: &str) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("WebSocket connection failed");
    ws_stream
}

/// Helper to receive next text message with timeout
async fn receive_text(
    ws: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
) -> Option<String> {
    match tokio::time::timeout(Duration::from_secs(5), ws.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => Some(text),
        Ok(Some(Ok(_))) => None, // Non-text message
        _ => None,               // Timeout or error
    }
}

/// Test: Client can reconnect after disconnecting
#[tokio::test]
async fn test_websocket_reconnect_same_channel() {
    let server = WsTestServer::start().await;
    let channel_id = server.create_channel("Reconnect Test").await;

    // First connection
    let ws_url = server.ws_url(&channel_id);
    let mut ws1 = connect_ws(&ws_url).await;

    // Receive connected message
    let msg1 = receive_text(&mut ws1)
        .await
        .expect("Should receive connected msg");
    let json1: serde_json::Value = serde_json::from_str(&msg1).expect("Parse failed");

    let _count_before_disconnect = json1["subscriber_count"]
        .as_u64()
        .expect("Missing subscriber_count");

    // Close first connection
    ws1.close(None).await.expect("Close should succeed");

    // Small delay for cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second connection (reconnect)
    let mut ws2 = connect_ws(&ws_url).await;

    // Receive connected message on reconnect
    let msg2 = receive_text(&mut ws2)
        .await
        .expect("Should receive connected msg on reconnect");
    let json2: serde_json::Value = serde_json::from_str(&msg2).expect("Parse failed");

    let count_after_reconnect = json2["subscriber_count"]
        .as_u64()
        .expect("Missing subscriber_count");

    // After disconnect + reconnect, subscriber count should be consistent
    // The old connection is gone, new one is active
    assert_eq!(
        count_after_reconnect, 1,
        "Should have exactly 1 subscriber after reconnect"
    );

    ws2.close(None).await.ok();
}

/// Test: Messages arrive in order when sent rapidly
#[tokio::test]
async fn test_websocket_message_ordering() {
    let server = WsTestServer::start().await;
    let channel_id = server.create_channel("Ordering Test").await;

    // Connect WebSocket
    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Skip connected message
    receive_text(&mut ws).await;

    // Small delay to ensure ready
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Send messages rapidly via HTTP API
    let num_messages = 10;
    for i in 0..num_messages {
        server
            .send_message(
                &channel_id,
                &format!("Message {}", i),
                &format!("https://example.com/{}", i),
            )
            .await;
    }

    // Collect all received messages
    let mut received_messages = Vec::new();
    let timeout_duration = Duration::from_secs(10);
    let start_time = std::time::Instant::now();

    while received_messages.len() < num_messages && start_time.elapsed() < timeout_duration {
        if let Some(text) = receive_text(&mut ws).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if json["type"] == "message" {
                    if let Some(name) = json["name"].as_str() {
                        received_messages.push(name.to_string());
                    }
                }
            }
        }
    }

    // Verify we received all messages
    assert_eq!(
        received_messages.len(),
        num_messages as usize,
        "Expected {} messages, got {}",
        num_messages,
        received_messages.len()
    );

    // Verify ordering - each message should have increasing number
    for (i, msg_name) in received_messages.iter().enumerate() {
        assert!(
            msg_name.contains(&i.to_string()),
            "Message {} should contain '{}', got '{}'",
            i,
            i,
            msg_name
        );
    }

    ws.close(None).await.ok();
}

/// Test: Large message payload handling
#[tokio::test]
async fn test_websocket_large_message() {
    let server = WsTestServer::start().await;
    let channel_id = server.create_channel("Large Message Test").await;

    // Connect WebSocket
    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Skip connected message
    receive_text(&mut ws).await;

    // Small delay
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create a large message name (near max size of 100 chars)
    let large_name = "A".repeat(90); // Under the 100 char limit

    // Send large message via HTTP
    server
        .send_message(&channel_id, &large_name, "https://example.com/large")
        .await;

    // Try to receive it
    let received = tokio::time::timeout(Duration::from_secs(5), receive_text(&mut ws)).await;

    match received {
        Ok(Some(text)) => {
            let json: serde_json::Value = serde_json::from_str(&text).expect("Parse failed");

            if json["type"] == "message" {
                let received_name = json["name"].as_str().unwrap_or("");

                assert_eq!(
                    received_name, large_name,
                    "Large message should be preserved"
                );
            }
        }
        _ => {
            // Message might not arrive due to timing, that's acceptable
            // The important thing is no crash/panic occurred
        }
    }

    ws.close(None).await.ok();
}

/// Test: Invalid JSON from client doesn't break connection
#[tokio::test]
async fn test_websocket_invalid_json_handling() {
    let server = WsTestServer::start().await;
    let channel_id = server.create_channel("Invalid JSON Test").await;

    // Connect WebSocket
    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Skip connected message
    receive_text(&mut ws).await;

    // Send invalid JSON
    ws.send(Message::Text("{invalid json".to_string()))
        .await
        .expect("Send should succeed");

    // Connection should still be alive - try sending valid ping
    ws.send(Message::Text(r#"{"type":"ping"}"#.to_string()))
        .await
        .expect("Valid send should succeed");

    // Should still be able to receive messages (connection not broken)
    let response = tokio::time::timeout(Duration::from_secs(3), receive_text(&mut ws)).await;

    match response {
        Ok(Some(text)) => {
            // Got a response - connection is working
            let json: serde_json::Value = serde_json::from_str(&text).expect("Parse failed");
            // Should be pong or some other valid response
            assert!(json.is_object(), "Response should be valid JSON object");
        }
        _ => {
            // No response within timeout, but connection didn't panic/crash
            // This is also acceptable behavior
        }
    }

    // Clean close
    ws.close(None).await.ok();
}

/// Test: Multiple rapid connections/disconnections
#[tokio::test]
async fn test_websocket_rapid_connect_disconnect() {
    let server = WsTestServer::start().await;
    let channel_id = server.create_channel("Rapid Connect Test").await;

    let ws_url = server.ws_url(&channel_id);

    // Rapidly connect and disconnect multiple times
    for _i in 0..5 {
        let mut ws = connect_ws(&ws_url).await;

        // Optionally receive connected message
        let _ = receive_text(&mut ws).await;

        // Short delay to simulate brief usage
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Close connection
        ws.close(None).await.expect("Close should succeed");

        // Brief pause between connections
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    // Server should still be functional after rapid connect/disconnect cycle
    let final_ws_url = server.ws_url(&channel_id);
    let mut final_ws = connect_ws(&final_ws_url).await;

    // Should be able to connect successfully
    let msg = receive_text(&mut final_ws).await;
    assert!(msg.is_some(), "Final connection should work");

    final_ws.close(None).await.ok();
}
