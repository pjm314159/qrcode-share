//! Security headers middleware
//!
//! Adds security-related headers to all responses.

use axum::{
    body::Body,
    http::{Request, Response},
};
use tower::{Layer, Service};

/// Security headers layer
#[derive(Debug, Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersMiddleware { inner }
    }
}

/// Security headers middleware service
#[derive(Debug, Clone)]
pub struct SecurityHeadersMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for SecurityHeadersMiddleware<S>
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
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut response = inner.call(request).await?;

            // Add security headers
            let headers = response.headers_mut();
            headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
            headers.insert("X-Frame-Options", "DENY".parse().unwrap());
            headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
            headers.insert(
                "Referrer-Policy",
                "strict-origin-when-cross-origin".parse().unwrap(),
            );

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_headers_added() {
        let service = SecurityHeadersLayer.layer(tower::service_fn(|_req: Request<Body>| async {
            Ok::<_, std::convert::Infallible>(Response::new(Body::empty()))
        }));

        let request = Request::builder().body(Body::empty()).unwrap();
        let response = service.oneshot(request).await.unwrap();

        let headers = response.headers();
        assert_eq!(headers.get("X-Content-Type-Options").unwrap(), "nosniff");
        assert_eq!(headers.get("X-Frame-Options").unwrap(), "DENY");
        assert_eq!(headers.get("X-XSS-Protection").unwrap(), "1; mode=block");
        assert_eq!(
            headers.get("Referrer-Policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
    }
}
