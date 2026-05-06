//! Integration tests for QRcode Share Backend
//!
//! These tests verify end-to-end functionality of the API.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use reqwest::Client;
use tokio::net::TcpListener;

use qrcode_share_backend::{build_router, AppState, Config};

/// Test server wrapper
struct TestServer {
    addr: SocketAddr,
    client: Client,
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
            if let Err(e) = axum::serve(listener, router).await {
                eprintln!("Server error: {}", e);
            }
        });

        // Wait for server to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self {
            addr,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    /// Get the server base URL
    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Test: Server health check endpoint
#[tokio::test]
async fn test_health_check() {
    let server = TestServer::start().await;

    let response = server
        .client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test: Metrics endpoint
#[tokio::test]
async fn test_metrics_endpoint() {
    let server = TestServer::start().await;

    let response = server
        .client
        .get(format!("{}/metrics", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test: Create channel endpoint
#[tokio::test]
async fn test_create_channel() {
    let server = TestServer::start().await;

    let body = serde_json::json!({
        "name": "Test Channel",
        "password": null,
        "link_limitation": null
    });

    let response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

/// Test: List channels endpoint
#[tokio::test]
async fn test_list_channels() {
    let server = TestServer::start().await;

    let response = server
        .client
        .get(format!("{}/api/channels", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test: Get non-existent channel
#[tokio::test]
async fn test_get_channel_not_found() {
    let server = TestServer::start().await;

    let response = server
        .client
        .get(format!("{}/api/channels/nonexistent", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test: Create channel with invalid data
#[tokio::test]
async fn test_create_channel_invalid() {
    let server = TestServer::start().await;

    let body = serde_json::json!({
        "name": "",  // Empty name should fail validation
    });

    let response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Test: Full producer flow — create channel → send message
#[tokio::test]
async fn test_producer_flow() {
    let server = TestServer::start().await;

    // Step 1: Create channel
    let create_body = serde_json::json!({
        "name": "Producer Test Channel",
    });

    let create_response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&create_body)
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Parse response to get channel ID
    let create_result: serde_json::Value = create_response.json().await.unwrap();
    let channel_id = create_result["data"]["id"].as_str().unwrap();

    // Step 2: Send message to channel
    let message_body = serde_json::json!({
        "name": "Test Message",
        "link": "https://example.com",
        "expire_seconds": 3600
    });

    let message_response = server
        .client
        .post(format!(
            "{}/api/channels/{}/messages",
            server.base_url(),
            channel_id
        ))
        .json(&message_body)
        .send()
        .await
        .unwrap();

    assert_eq!(message_response.status(), StatusCode::CREATED);
}

/// Test: Consumer flow — create channel → get messages
#[tokio::test]
async fn test_consumer_flow() {
    let server = TestServer::start().await;

    // Step 1: Create channel
    let create_body = serde_json::json!({
        "name": "Consumer Test Channel",
    });

    let create_response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&create_body)
        .send()
        .await
        .unwrap();

    let create_result: serde_json::Value = create_response.json().await.unwrap();
    let channel_id = create_result["data"]["id"].as_str().unwrap();

    // Step 2: Get messages (should be empty initially)
    let messages_response = server
        .client
        .get(format!(
            "{}/api/channels/{}/messages",
            server.base_url(),
            channel_id
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(messages_response.status(), StatusCode::OK);

    let messages_result: serde_json::Value = messages_response.json().await.unwrap();

    assert!(messages_result["success"].as_bool().unwrap());
    assert!(messages_result["data"]["messages"]
        .as_array()
        .unwrap()
        .is_empty());
}

/// Test: Link limitation flow
#[tokio::test]
async fn test_link_limitation_flow() {
    let server = TestServer::start().await;

    // Step 1: Create channel with link limitation
    let create_body = serde_json::json!({
        "name": "Limited Channel",
        "link_limitation": ["allowed.com"]
    });

    let create_response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&create_body)
        .send()
        .await
        .unwrap();

    let create_result: serde_json::Value = create_response.json().await.unwrap();
    let channel_id = create_result["data"]["id"].as_str().unwrap();

    // Step 2: Send allowed link
    let allowed_body = serde_json::json!({
        "name": "Allowed Link",
        "link": "https://allowed.com/page",
        "expire_seconds": 3600
    });

    let allowed_response = server
        .client
        .post(format!(
            "{}/api/channels/{}/messages",
            server.base_url(),
            channel_id
        ))
        .json(&allowed_body)
        .send()
        .await
        .unwrap();

    assert_eq!(allowed_response.status(), StatusCode::CREATED);

    // Step 3: Send disallowed link
    let disallowed_body = serde_json::json!({
        "name": "Disallowed Link",
        "link": "https://disallowed.com/page",
        "expire_seconds": 3600
    });

    let disallowed_response = server
        .client
        .post(format!(
            "{}/api/channels/{}/messages",
            server.base_url(),
            channel_id
        ))
        .json(&disallowed_body)
        .send()
        .await
        .unwrap();

    assert_eq!(disallowed_response.status(), StatusCode::BAD_REQUEST);
}

/// Test: Delete channel
#[tokio::test]
async fn test_delete_channel() {
    let server = TestServer::start().await;

    // Create channel
    let create_body = serde_json::json!({
        "name": "Channel to Delete",
    });

    let create_response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&create_body)
        .send()
        .await
        .unwrap();

    let create_result: serde_json::Value = create_response.json().await.unwrap();
    let channel_id = create_result["data"]["id"].as_str().unwrap();

    // Delete channel
    let delete_response = server
        .client
        .delete(format!("{}/api/channels/{}", server.base_url(), channel_id))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    // Verify channel is deleted
    let get_response = server
        .client
        .get(format!("{}/api/channels/{}", server.base_url(), channel_id))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}
