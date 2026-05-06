//! Rate limiting middleware
//!
//! Provides rate limiting for API endpoints with proper headers.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use dashmap::DashMap;
use smallvec::SmallVec;
use tower::{Layer, Service};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

/// Rate limit layer for axum
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
    limiter: Arc<RateLimiter>,
}

impl RateLimitLayer {
    /// Create a new rate limit layer
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            limiter: Arc::new(RateLimiter::new()),
        }
    }

    /// Create with default configuration
    pub fn per_minute(max_requests: usize) -> Self {
        Self::new(RateLimitConfig {
            max_requests,
            window: Duration::from_secs(60),
        })
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            config: self.config.clone(),
            limiter: self.limiter.clone(),
        }
    }
}

/// Rate limiting middleware service
#[derive(Debug, Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    config: RateLimitConfig,
    limiter: Arc<RateLimiter>,
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let limiter = self.limiter.clone();
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get client identifier (IP address or API key)
            let client_id = get_client_id(&request);

            // Check rate limit
            let (allowed, remaining, reset_after) =
                limiter.check(&client_id, config.max_requests, config.window);

            if !allowed {
                // Build rate limit exceeded response
                let body = serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "RATE_LIMIT_EXCEEDED",
                        "message": "Rate limit exceeded. Please try again later.",
                        "retry_after_seconds": reset_after.as_secs()
                    }
                });

                let response = Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("X-RateLimit-Limit", config.max_requests)
                    .header("X-RateLimit-Remaining", 0)
                    .header("X-RateLimit-Reset", reset_after.as_secs())
                    .header("Retry-After", reset_after.as_secs())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap();

                return Ok(response);
            }

            // Call inner service
            let mut response = inner.call(request).await?;

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                config.max_requests.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-RateLimit-Remaining",
                remaining.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-RateLimit-Reset",
                reset_after.as_secs().to_string().parse().unwrap(),
            );

            Ok(response)
        })
    }
}

/// In-memory rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    requests: DashMap<String, SmallVec<[Instant; 8]>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }

    /// Check if a request is allowed
    pub fn check(
        &self,
        client_id: &str,
        max_requests: usize,
        window: Duration,
    ) -> (bool, usize, Duration) {
        let now = Instant::now();
        let cutoff = now - window;

        let mut entry = self.requests.entry(client_id.to_string()).or_default();

        // Remove expired timestamps
        entry.retain(|timestamp| *timestamp > cutoff);

        let current_count = entry.len();

        if current_count < max_requests {
            // Allow request
            entry.push(now);
            let remaining = max_requests - current_count - 1;
            let reset_after = window;
            (true, remaining, reset_after)
        } else {
            // Deny request
            let oldest = entry.first().copied().unwrap_or(now);
            let reset_after = oldest + window - now;
            (false, 0, reset_after)
        }
    }

    /// Clean up stale entries
    #[allow(dead_code)]
    pub fn cleanup_stale(&self) {
        let now = Instant::now();
        self.requests.retain(|_, timestamps| {
            timestamps.retain(|timestamp| now - *timestamp < Duration::from_secs(3600));
            !timestamps.is_empty()
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Get client identifier from request
fn get_client_id<B>(request: &Request<B>) -> String {
    // Try to get from X-Forwarded-For header
    if let Some(forwarded_for) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try to get from X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Default to unknown
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new();
        let client_id = "test_client";

        for i in 0..5 {
            let (allowed, remaining, _) = limiter.check(client_id, 10, Duration::from_secs(60));
            assert!(allowed, "Request {} should be allowed", i);
            assert_eq!(remaining, 9 - i);
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new();
        let client_id = "test_client";

        // Use up all requests
        for _ in 0..5 {
            limiter.check(client_id, 5, Duration::from_secs(60));
        }

        // Next request should be blocked
        let (allowed, remaining, _) = limiter.check(client_id, 5, Duration::from_secs(60));
        assert!(!allowed);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_rate_limiter_independent_clients() {
        let limiter = RateLimiter::new();

        // Client 1 uses up their limit
        for _ in 0..5 {
            limiter.check("client1", 5, Duration::from_secs(60));
        }

        // Client 2 should still be allowed
        let (allowed, _, _) = limiter.check("client2", 5, Duration::from_secs(60));
        assert!(allowed);
    }

    #[test]
    fn test_rate_limit_layer_creation() {
        let layer = RateLimitLayer::per_minute(100);
        assert_eq!(layer.config.max_requests, 100);
    }

    #[test]
    fn test_get_client_id_unknown() {
        let request = Request::builder().body(Body::empty()).unwrap();
        let client_id = get_client_id(&request);
        assert_eq!(client_id, "unknown");
    }

    #[test]
    fn test_get_client_id_from_forwarded_for() {
        let request = Request::builder()
            .header("X-Forwarded-For", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let client_id = get_client_id(&request);
        assert_eq!(client_id, "192.168.1.1");
    }

    #[test]
    fn test_get_client_id_from_real_ip() {
        let request = Request::builder()
            .header("X-Real-IP", "192.168.1.2")
            .body(Body::empty())
            .unwrap();
        let client_id = get_client_id(&request);
        assert_eq!(client_id, "192.168.1.2");
    }
}
