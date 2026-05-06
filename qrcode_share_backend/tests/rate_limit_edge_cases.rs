//! Rate limiting edge case tests

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use reqwest::Client;

use qrcode_share_backend::{build_router, AppState, Config, RateLimiter};

struct RateLimitTestServer {
    addr: std::net::SocketAddr,
    client: Client,
}

impl RateLimitTestServer {
    async fn start() -> Self {
        let config = Arc::new(Config::from_env().expect("Config failed"));
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

    async fn create_channel_request(&self, name: &str) -> (StatusCode, Option<serde_json::Value>) {
        let body = serde_json::json!({ "name": name });

        let response = self
            .client
            .post(format!("{}/api/channels", self.base_url()))
            .json(&body)
            .send()
            .await
            .expect("Request failed");

        let status = response.status();

        if status == StatusCode::CREATED || status == StatusCode::OK {
            let json: serde_json::Value = response.json().await.unwrap_or_default();
            (status, Some(json))
        } else {
            (status, None)
        }
    }
}

/// Test: Burst requests within limit should be allowed
#[tokio::test]
async fn test_rate_limit_burst_allowed() {
    let limiter = RateLimiter::new(10, Duration::from_secs(60));

    let ip = "192.168.1.100";

    for i in 0..5 {
        let result = limiter.check(ip);
        assert!(result, "Burst request {} should be allowed", i + 1);
    }

    let remaining = limiter.remaining(ip);
    assert!(remaining > 0, "Should have remaining requests after burst");
}

/// Test: Rate limit window resets after time passes
#[tokio::test]
async fn test_rate_limit_window_reset() {
    let limiter = RateLimiter::new(3, Duration::from_millis(100));

    let ip = "192.168.1.200";

    assert!(limiter.check(ip), "Request 1 should be allowed");
    assert!(limiter.check(ip), "Request 2 should be allowed");
    assert!(limiter.check(ip), "Request 3 should be allowed");
    assert!(!limiter.check(ip), "Request 4 should be blocked");

    tokio::time::sleep(Duration::from_millis(150)).await;

    assert!(
        limiter.check(ip),
        "Should be able to make request after window reset"
    );

    let remaining_after_reset = limiter.remaining(ip);
    assert!(
        remaining_after_reset >= 2,
        "Should have remaining quota after reset, got {}",
        remaining_after_reset
    );
}

/// Test: Multiple independent clients have separate rate limits
#[tokio::test]
async fn test_rate_limit_independent_clients() {
    let limiter = RateLimiter::new(5, Duration::from_secs(60));

    let client_a = "10.0.0.1";
    let client_b = "10.0.0.2";

    assert!(limiter.check(client_a), "Client A - request 1");
    assert!(limiter.check(client_a), "Client A - request 2");
    assert!(limiter.check(client_a), "Client A - request 3");
    assert!(limiter.check(client_a), "Client A - request 4");
    assert!(limiter.check(client_a), "Client A - request 5");
    assert!(!limiter.check(client_a), "Client A should now be limited");

    assert!(
        limiter.check(client_b),
        "Client B should have independent rate limit and be allowed"
    );

    let client_b_remaining = limiter.remaining(client_b);
    assert!(
        client_b_remaining >= 4,
        "Client B should have most of their quota remaining, got {}",
        client_b_remaining
    );
}

/// Test: HTTP API rate limiting integration
#[tokio::test]
async fn test_api_rate_limiting_integration() {
    let server = RateLimitTestServer::start().await;

    let mut success_count = 0u32;

    for i in 0..20 {
        let (status, _) = server
            .create_channel_request(&format!("Channel {}", i))
            .await;

        if status == StatusCode::CREATED {
            success_count += 1;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    assert!(
        success_count > 0,
        "At least some requests should succeed, got {} successes",
        success_count
    );
}

/// Test: Different endpoints may have different rate limits
#[tokio::test]
async fn test_different_endpoints_rate_limits() {
    let server = RateLimitTestServer::start().await;

    let (channel_status, _) = server.create_channel_request("Test Channel").await;
    assert_eq!(
        channel_status,
        StatusCode::CREATED,
        "Channel creation should work"
    );

    let health_response = server
        .client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await
        .expect("Health request failed");

    assert_eq!(
        health_response.status(),
        StatusCode::OK,
        "Health endpoint should always work"
    );
}
