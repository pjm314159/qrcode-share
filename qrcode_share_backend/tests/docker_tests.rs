//! Docker integration tests for QRcode Share Backend
//!
//! These tests use testcontainers to spin up a PostgreSQL container
//! for real database integration testing.

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use reqwest::Client;
use serial_test::serial;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use qrcode_share_backend::{build_router, AppState, Config};

/// Check if Docker is available for testcontainers
async fn docker_available() -> bool {
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::process::Command::new("docker").arg("info").output(),
    )
    .await
    {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    }
}

/// Test context that manages the test container and provides
/// a configured test server
struct TestContext {
    _postgres: testcontainers::ContainerAsync<Postgres>,
    config: Arc<Config>,
    client: Client,
}

impl TestContext {
    /// Create a new test context with a PostgreSQL container.
    /// Returns None if the container cannot be started within timeout.
    async fn try_new() -> Option<Self> {
        let start_result = tokio::time::timeout(
            Duration::from_secs(60),
            Postgres::default()
                .with_db_name("qrcode_share_test")
                .with_user("test_user")
                .with_password("test_password")
                .start(),
        )
        .await;

        let postgres = match start_result {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                eprintln!("Failed to start PostgreSQL container: {}", e);
                return None;
            }
            Err(_) => {
                eprintln!("Timeout starting PostgreSQL container");
                return None;
            }
        };

        let host = postgres.get_host().await.ok()?;
        let port = postgres.get_host_port_ipv4(5432).await.ok()?;

        let database_url = format!(
            "postgres://test_user:test_password@{}:{}/qrcode_share_test",
            host, port
        );

        let mut config = Config::from_env().expect("Failed to load config");
        config.database_url = database_url.clone();
        config.db_max_connections = 3;
        config.db_min_connections = 1;
        config.max_channels = 100;
        config.max_messages_per_channel = 50;
        let config = Arc::new(config);

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Some(Self {
            _postgres: postgres,
            config,
            client,
        })
    }

    /// Start a test server and return its address
    async fn start_server(&self) -> (String, tokio::task::JoinHandle<()>) {
        let app_state = AppState::new(self.config.clone());
        let router = build_router(app_state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind port");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, router).await {
                eprintln!("Server error: {}", e);
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        (base_url, handle)
    }
}

/// Test: Database connection with testcontainers
#[tokio::test]
#[serial]
async fn test_database_connection() {
    if !docker_available().await {
        eprintln!("Skipping test_database_connection: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_database_connection: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let response = ctx
        .client
        .get(format!("{}/health", base_url))
        .send()
        .await
        .expect("Failed to send health check request");

    assert_eq!(response.status(), StatusCode::OK);
}

/// Test: Create channel with database persistence
#[tokio::test]
#[serial]
async fn test_create_channel_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_create_channel_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_create_channel_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let body = serde_json::json!({
        "name": "Docker Test Channel",
        "password": null,
        "link_limitation": null
    });

    let response = ctx
        .client
        .post(format!("{}/api/channels", base_url))
        .json(&body)
        .send()
        .await
        .expect("Failed to create channel");

    assert_eq!(response.status(), StatusCode::CREATED);

    let result: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(
        result["success"].as_bool().expect("Missing success field"),
        "Expected success to be true"
    );
    assert!(
        result["data"]["id"].as_str().is_some(),
        "Expected channel id"
    );
    assert_eq!(
        result["data"]["name"].as_str().expect("Missing name field"),
        "Docker Test Channel"
    );
}

/// Test: Full message flow with database
#[tokio::test]
#[serial]
async fn test_message_flow_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_message_flow_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_message_flow_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let create_body = serde_json::json!({
        "name": "Message Flow Test"
    });

    let create_response = ctx
        .client
        .post(format!("{}/api/channels", base_url))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create channel");

    let create_result: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse create response");
    let channel_id = create_result["data"]["id"]
        .as_str()
        .expect("Missing channel id");

    let message_body = serde_json::json!({
        "name": "Test Message",
        "link": "https://example.com",
        "expire_seconds": 3600
    });

    let message_response = ctx
        .client
        .post(format!("{}/api/channels/{}/messages", base_url, channel_id))
        .json(&message_body)
        .send()
        .await
        .expect("Failed to send message");

    assert_eq!(message_response.status(), StatusCode::CREATED);

    let get_response = ctx
        .client
        .get(format!("{}/api/channels/{}/messages", base_url, channel_id))
        .send()
        .await
        .expect("Failed to get messages");

    assert_eq!(get_response.status(), StatusCode::OK);

    let get_result: serde_json::Value = get_response
        .json()
        .await
        .expect("Failed to parse messages response");
    assert!(
        get_result["success"]
            .as_bool()
            .expect("Missing success field"),
        "Expected success to be true"
    );
    let messages = get_result["data"]["messages"]
        .as_array()
        .expect("Missing messages array");
    assert_eq!(messages.len(), 1, "Expected 1 message");
}

/// Test: Link limitation with database
#[tokio::test]
#[serial]
async fn test_link_limitation_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_link_limitation_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_link_limitation_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let create_body = serde_json::json!({
        "name": "Limited Channel",
        "link_limitation": ["allowed.com", "example.org"]
    });

    let create_response = ctx
        .client
        .post(format!("{}/api/channels", base_url))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create channel");

    let create_result: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let channel_id = create_result["data"]["id"]
        .as_str()
        .expect("Missing channel id");

    let allowed_body = serde_json::json!({
        "name": "Allowed",
        "link": "https://allowed.com/page",
        "expire_seconds": 3600
    });

    let allowed_response = ctx
        .client
        .post(format!("{}/api/channels/{}/messages", base_url, channel_id))
        .json(&allowed_body)
        .send()
        .await
        .expect("Failed to send allowed message");

    assert_eq!(allowed_response.status(), StatusCode::CREATED);

    let disallowed_body = serde_json::json!({
        "name": "Disallowed",
        "link": "https://disallowed.com/page",
        "expire_seconds": 3600
    });

    let disallowed_response = ctx
        .client
        .post(format!("{}/api/channels/{}/messages", base_url, channel_id))
        .json(&disallowed_body)
        .send()
        .await
        .expect("Failed to send disallowed message");

    assert_eq!(disallowed_response.status(), StatusCode::BAD_REQUEST);
}

/// Test: Channel deletion with database
#[tokio::test]
#[serial]
async fn test_delete_channel_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_delete_channel_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_delete_channel_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let create_body = serde_json::json!({
        "name": "To Delete"
    });

    let create_response = ctx
        .client
        .post(format!("{}/api/channels", base_url))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create channel");

    let create_result: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let channel_id = create_result["data"]["id"]
        .as_str()
        .expect("Missing channel id");

    let delete_response = ctx
        .client
        .delete(format!("{}/api/channels/{}", base_url, channel_id))
        .send()
        .await
        .expect("Failed to delete channel");

    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_response = ctx
        .client
        .get(format!("{}/api/channels/{}", base_url, channel_id))
        .send()
        .await
        .expect("Failed to get deleted channel");

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

/// Test: Concurrent channel creation
#[tokio::test]
#[serial]
async fn test_concurrent_channels_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_concurrent_channels_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_concurrent_channels_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    let mut handles = vec![];

    for i in 0..5 {
        let url = base_url.clone();
        let client = ctx.client.clone();
        let handle = tokio::spawn(async move {
            let body = serde_json::json!({
                "name": format!("Concurrent Channel {}", i)
            });

            let response = client
                .post(format!("{}/api/channels", url))
                .json(&body)
                .send()
                .await
                .expect("Failed to send concurrent request");

            response.status()
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    for result in results {
        let status = result.expect("Task panicked");
        assert_eq!(status, StatusCode::CREATED);
    }

    let list_response = ctx
        .client
        .get(format!("{}/api/channels", base_url))
        .send()
        .await
        .expect("Failed to list channels");

    let list_result: serde_json::Value = list_response
        .json()
        .await
        .expect("Failed to parse list response");
    let total = list_result["data"]["total"]
        .as_i64()
        .expect("Missing total field");
    assert!(total >= 5, "Expected at least 5 channels, got {}", total);
}

/// Test: Metrics endpoint with database
#[tokio::test]
#[serial]
async fn test_metrics_docker() {
    if !docker_available().await {
        eprintln!("Skipping test_metrics_docker: Docker not available");
        return;
    }

    let Some(ctx) = TestContext::try_new().await else {
        eprintln!("Skipping test_metrics_docker: Could not start PostgreSQL container");
        return;
    };
    let (base_url, _server) = ctx.start_server().await;

    for i in 0..3 {
        let body = serde_json::json!({
            "name": format!("Metrics Test {}", i)
        });

        ctx.client
            .post(format!("{}/api/channels", base_url))
            .json(&body)
            .send()
            .await
            .expect("Failed to create channel for metrics test");
    }

    let response = ctx
        .client
        .get(format!("{}/metrics", base_url))
        .send()
        .await
        .expect("Failed to get metrics");

    assert_eq!(response.status(), StatusCode::OK);

    let result: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse metrics response");

    let channels_total = result["data"]["channels"]["total"]
        .as_i64()
        .expect("Missing channels.total field in metrics");
    assert!(
        channels_total >= 3,
        "Expected at least 3 channels in metrics, got {}",
        channels_total
    );
}
