//! WebSocket integration tests for QRcode Share Backend
//!
//! These tests verify WebSocket functionality end-to-end.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use qrcode_share_backend::{build_router, AppState, Config};

/// Test server wrapper
struct TestServer {
    addr: SocketAddr,
}

impl TestServer {
    /// Create and start a test server
    async fn start() -> Self {
        let config = Arc::new(Config::from_env().unwrap());
        let app_state = AppState::new(config);
        let router = build_router(app_state);

        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn the server
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        // Wait for server to start
        tokio::time::sleep(Duration::from_millis(200)).await;

        Self { addr }
    }

    /// Get the server address
    fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get WebSocket URL for a channel
    fn ws_url(&self, channel_id: &str) -> String {
        format!("ws://{}/api/channels/{}/ws", self.addr(), channel_id)
    }

    /// Get HTTP URL
    fn http_url(&self) -> String {
        format!("http://{}", self.addr())
    }
}

/// Connect to WebSocket
async fn connect_ws(url: &str) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    let (ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    ws
}

/// Create a channel via HTTP API
async fn create_channel(server: &TestServer, name: &str) -> String {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "name": name,
    });

    let response = client
        .post(format!("{}/api/channels", server.http_url()))
        .json(&body)
        .send()
        .await
        .unwrap();

    let result: serde_json::Value = response.json().await.unwrap();
    result["data"]["id"].as_str().unwrap().to_string()
}

/// Send a message to a channel via HTTP API
async fn send_message(server: &TestServer, channel_id: &str, name: &str, link: &str) {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "name": name,
        "link": link,
        "expire_seconds": 3600
    });

    client
        .post(format!(
            "{}/api/channels/{}/messages",
            server.http_url(),
            channel_id
        ))
        .json(&body)
        .send()
        .await
        .unwrap();
}

/// Test: WebSocket connects to channel and receives connected message
#[tokio::test]
async fn test_websocket_connect() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "WS Test Channel").await;

    // Connect to WebSocket
    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Receive connected message
    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("Timeout waiting for message")
        .expect("No message received")
        .expect("Error receiving message");

    let text = msg.to_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(json["type"], "connected");
    assert_eq!(json["channel_id"], channel_id);
    assert!(json["subscriber_count"].as_u64().unwrap() >= 1);
}

/// Test: WebSocket receives broadcast message
#[tokio::test]
async fn test_websocket_broadcast() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "Broadcast Test").await;

    // Connect to WebSocket
    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Skip connected message
    let _ = ws.next().await;

    // Small delay to ensure WebSocket is ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send message via HTTP API
    send_message(&server, &channel_id, "Test Message", "https://example.com").await;

    // Small delay for broadcast
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Receive broadcast message
    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("Timeout waiting for broadcast")
        .expect("No message received")
        .expect("Error receiving message");

    // Handle different message types
    match msg {
        Message::Text(text) => {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "message");
            assert_eq!(json["name"], "Test Message");
            assert_eq!(json["link"], "https://example.com");
        }
        Message::Ping(_) => {
            // Skip ping and try again
            let msg2 = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await
                .expect("Timeout waiting for broadcast")
                .expect("No message received")
                .expect("Error receiving message");
            let text = msg2.to_text().unwrap();
            let json: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(json["type"], "message");
        }
        _ => panic!("Unexpected message type: {:?}", msg),
    }
}

/// Test: Multiple WebSocket clients receive same broadcast
#[tokio::test]
async fn test_websocket_multiple_clients() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "Multi Client Test").await;

    // Connect two WebSocket clients
    let ws_url = server.ws_url(&channel_id);
    let mut ws1 = connect_ws(&ws_url).await;
    let mut ws2 = connect_ws(&ws_url).await;

    // Skip connected messages
    let _ = ws1.next().await;
    let _ = ws2.next().await;

    // Small delay to ensure WebSockets are ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send message via HTTP API
    send_message(&server, &channel_id, "Broadcast", "https://test.com").await;

    // Small delay for broadcast
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Helper to receive and parse message, skipping non-broadcast messages
    async fn receive_broadcast(
        ws: &mut WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> serde_json::Value {
        loop {
            let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await
                .expect("Timeout")
                .expect("No message")
                .expect("Error");

            let json: serde_json::Value = match msg {
                Message::Text(text) => serde_json::from_str(&text).unwrap(),
                Message::Ping(_) => continue,
                _ => panic!("Unexpected message type: {:?}", msg),
            };

            if json["type"] == "message" {
                return json;
            }
        }
    }

    let json1 = receive_broadcast(&mut ws1).await;
    let json2 = receive_broadcast(&mut ws2).await;

    assert_eq!(json1["name"], "Broadcast");
    assert_eq!(json2["name"], "Broadcast");
}

/// Test: WebSocket ping/pong heartbeat
#[tokio::test]
async fn test_websocket_heartbeat() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "Heartbeat Test").await;

    let ws_url = server.ws_url(&channel_id);
    let mut ws = connect_ws(&ws_url).await;

    // Skip connected message
    let _ = ws.next().await;

    // Small delay to ensure WebSocket is ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send ping
    let ping = serde_json::json!({"type": "ping"});
    ws.send(Message::Text(ping.to_string()))
        .await
        .expect("Failed to send ping");

    // Receive pong
    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("Timeout waiting for pong")
        .expect("No message received")
        .expect("Error receiving message");

    // Handle different message types
    match msg {
        Message::Text(text) => {
            let json: serde_json::Value =
                serde_json::from_str(&text).expect("Failed to parse pong response");
            assert_eq!(
                json["type"], "pong",
                "Expected pong response, got: {:?}",
                json
            );
        }
        Message::Ping(_) | Message::Pong(_) => {
            // Skip ping/pong frames and try again
            let msg2 = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await
                .expect("Timeout waiting for pong")
                .expect("No message received")
                .expect("Error receiving message");
            let text = msg2.to_text().expect("Expected text message");
            let json: serde_json::Value =
                serde_json::from_str(text).expect("Failed to parse pong response");
            assert_eq!(json["type"], "pong", "Expected pong response");
        }
        Message::Close(_) => {
            panic!("WebSocket closed unexpectedly");
        }
        _ => {
            panic!("Unexpected message type: {:?}", msg);
        }
    }
}

/// Test: WebSocket subscriber count updates
#[tokio::test]
async fn test_websocket_subscriber_count() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "Subscriber Test").await;

    let ws_url = server.ws_url(&channel_id);

    // First client connects
    let mut ws1 = connect_ws(&ws_url).await;
    let msg1 = ws1.next().await.unwrap().unwrap();
    let json1: serde_json::Value = serde_json::from_str(msg1.to_text().unwrap()).unwrap();
    let count1 = json1["subscriber_count"].as_u64().unwrap();

    // Second client connects
    let mut ws2 = connect_ws(&ws_url).await;
    let msg2 = ws2.next().await.unwrap().unwrap();
    let json2: serde_json::Value = serde_json::from_str(msg2.to_text().unwrap()).unwrap();
    let count2 = json2["subscriber_count"].as_u64().unwrap();

    // Subscriber count should increase
    assert!(
        count2 > count1,
        "count2 ({}) should be greater than count1 ({})",
        count2,
        count1
    );

    // Close first connection
    ws1.close(None).await.ok();

    // Wait longer for cleanup (the WebSocket handler needs time to detect disconnection)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Third client should see decreased count
    let mut ws3 = connect_ws(&ws_url).await;
    let msg3 = ws3.next().await.unwrap().unwrap();
    let json3: serde_json::Value = serde_json::from_str(msg3.to_text().unwrap()).unwrap();
    let count3 = json3["subscriber_count"].as_u64().unwrap();

    // The count should be less than count2 (but might still be >= count1 due to timing)
    assert!(
        count3 <= count2,
        "count3 ({}) should be less than or equal to count2 ({})",
        count3,
        count2
    );
}

/// Test: Load test with concurrent WebSocket connections
#[tokio::test]
async fn test_websocket_concurrent_connections() {
    let server = TestServer::start().await;
    let channel_id = create_channel(&server, "Load Test").await;

    let ws_url = server.ws_url(&channel_id);
    let num_clients = 10;

    // Connect multiple clients concurrently
    let mut handles = vec![];

    for _ in 0..num_clients {
        let url = ws_url.clone();
        let handle = tokio::spawn(async move {
            let mut ws = connect_ws(&url).await;

            // Receive connected message
            let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await
                .expect("Timeout")
                .expect("No message")
                .expect("Error");

            let json: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
            assert_eq!(json["type"], "connected");

            ws
        });
        handles.push(handle);
    }

    // Wait for all connections
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All connections should succeed
    assert_eq!(results.len(), num_clients);
    for result in results {
        assert!(result.is_ok());
    }
}
