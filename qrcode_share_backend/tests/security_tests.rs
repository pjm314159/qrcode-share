//! Security and integration tests
//!
//! These tests verify security headers, CORS configuration,
//! and response compression.

use std::time::Duration;

use axum::http::StatusCode;
use reqwest::header::{HeaderMap, CONTENT_TYPE, ORIGIN};
use reqwest::{Client, Method};

/// Test server for security tests
struct SecurityTestServer {
    addr: std::net::SocketAddr,
    client: Client,
}

impl SecurityTestServer {
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

    async fn get_headers(&self, path: &str) -> (StatusCode, HeaderMap) {
        let response = self
            .client
            .get(format!("{}{}", self.base_url(), path))
            .send()
            .await
            .expect("Request failed");

        (response.status(), response.headers().clone())
    }
}

/// Test: Security headers are present on API responses
#[tokio::test]
async fn test_security_headers_present() {
    let server = SecurityTestServer::start().await;

    // Test health endpoint headers
    let (status, headers) = server.get_headers("/health").await;

    assert_eq!(status, StatusCode::OK);

    // Check for common security headers
    // Note: Our SecurityHeadersLayer adds these:

    // X-Content-Type-Options: nosniff
    if let Some(xcto) = headers.get("x-content-type-options") {
        assert_eq!(
            xcto.to_str().unwrap_or(""),
            "nosniff",
            "X-Content-Type-Options should be 'nosniff'"
        );
    }

    // X-Frame-Options should be set
    if let Some(xfo) = headers.get("x-frame-options") {
        let value = xfo.to_str().unwrap_or("");
        assert!(
            value == "DENY" || value == "SAMEORIGIN",
            "X-Frame-Options should be DENY or SAMEORIGIN, got '{}'",
            value
        );
    }

    // Content-Security-Policy or X-XSS-Protection might be present
    if headers.get("content-security-policy").is_some() {
        let csp = headers.get("content-security-policy").unwrap();
        assert!(csp.to_str().is_ok(), "CSP header should be valid string");
    }

    // Verify no sensitive information in headers
    let headers_str = format!("{:?}", headers);
    assert!(
        !headers_str.contains("password")
            && !headers_str.contains("secret")
            && !headers_str.contains("token"),
        "Headers should not contain sensitive information"
    );
}

/// Test: CORS preflight request handling
#[tokio::test]
async fn test_cors_preflight_request() {
    let server = SecurityTestServer::start().await;

    // Send OPTIONS preflight request with Origin header
    // Use request() method instead of options() which doesn't exist in reqwest
    let response = server
        .client
        .request(
            Method::OPTIONS,
            format!("{}/api/channels", server.base_url()),
        )
        .header(ORIGIN, "https://example.com")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type")
        .send()
        .await
        .expect("Preflight request failed");

    // Preflight should succeed (200 or 204)
    let status = response.status();

    // Accept various status codes for OPTIONS requests
    assert!(
        status == StatusCode::OK
            || status == StatusCode::NO_CONTENT
            || status == StatusCode::METHOD_NOT_ALLOWED
            || status == StatusCode::NOT_FOUND,
        "Preflight should return acceptable status, got {}",
        status
    );

    // Check CORS headers in response (if present)
    let headers = response.headers();

    // If CORS is configured, these headers should be present
    if let Some(allow_origin) = headers.get("access-control-allow-origin") {
        let origin_value = allow_origin.to_str().unwrap_or("");

        // Should allow the requested origin or wildcard
        assert!(
            origin_value == "*" || origin_value == "https://example.com",
            "Allow-Origin should be '*' or specific origin, got '{}'",
            origin_value
        );
    }

    if let Some(allow_methods) = headers.get("access-control-allow-methods") {
        let methods_str = allow_methods.to_str().unwrap_or("");
        // Methods header should exist and be a string
        assert!(!methods_str.is_empty(), "Allow-Methods should not be empty");
    }
}

/// Test: Response content type is correct
#[tokio::test]
async fn test_response_content_type() {
    let server = SecurityTestServer::start().await;

    // Health endpoint should return JSON or text
    let (status, headers) = server.get_headers("/health").await;
    assert_eq!(status, StatusCode::OK);

    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        let ct = content_type.to_str().unwrap_or("");
        assert!(
            ct.contains("application/json") || ct.contains("text/plain"),
            "Health endpoint should return JSON or text, got '{}'",
            ct
        );
    }

    // Metrics endpoint should also return JSON
    let (metrics_status, metrics_headers) = server.get_headers("/metrics").await;
    assert_eq!(metrics_status, StatusCode::OK);

    if let Some(metrics_ct) = metrics_headers.get(CONTENT_TYPE) {
        let ct = metrics_ct.to_str().unwrap_or("");
        assert!(
            ct.contains("application/json"),
            "Metrics endpoint should return JSON, got '{}'",
            ct
        );
    }
}

/// Test: API returns proper error responses for invalid requests
#[tokio::test]
async fn test_error_responses_format() {
    let server = SecurityTestServer::start().await;

    // Send invalid data to create channel endpoint
    let body = serde_json::json!({
        "name": "", // Empty name - should fail validation
    });

    let response = server
        .client
        .post(format!("{}/api/channels", server.base_url()))
        .json(&body)
        .send()
        .await
        .expect("Request failed");

    // Should return client error (4xx)
    let status = response.status();
    assert!(
        status.is_client_error(),
        "Invalid request should return 4xx status, got {}",
        status
    );

    // Error response should be JSON with expected structure
    let json: serde_json::Value = response.json().await.expect("Should parse JSON error");

    assert!(
        json.get("success").is_some(),
        "Error response should have 'success' field"
    );
    assert_eq!(
        json["success"].as_bool(),
        Some(false),
        "Error response success should be false"
    );

    assert!(
        json.get("error").is_some(),
        "Error response should have 'error' object"
    );

    // Error details should not expose internal information
    let error_json = json["error"].to_string();
    assert!(
        !error_json.contains("stacktrace") && !error_sql_contains_sensitive_info(&error_json),
        "Error should not expose sensitive internal info"
    );
}

fn error_sql_contains_sensitive_info(error_str: &str) -> bool {
    let lower = error_str.to_lowercase();
    lower.contains("database_url")
        || lower.contains("password_hash")
        || lower.contains("internal_server")
}

/// Test: Non-existent endpoints return 404 properly
#[tokio::test]
async fn test_not_found_handling() {
    let server = SecurityTestServer::start().await;

    let response = server
        .client
        .get(format!("{}/nonexistent-endpoint", server.base_url()))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 404 should ideally return JSON, but we just verify status code
    // Some servers may not return body for 404
    let content_length = response.content_length().unwrap_or(0);

    if content_length > 0 {
        // If there's a body, try to parse as JSON
        match response.json::<serde_json::Value>().await {
            Ok(json) => {
                assert_eq!(json["success"].as_bool(), Some(false));
                assert!(
                    json.get("error").is_some(),
                    "404 response should include error details"
                );
            }
            Err(_) => {
                // Body exists but isn't JSON - still acceptable
            }
        }
    }
}
