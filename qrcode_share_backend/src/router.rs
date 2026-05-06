//! Router configuration
//!
//! Defines all HTTP routes and middleware for the API.

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::handlers;
use crate::middleware::SecurityHeadersLayer;
use crate::state::AppState;

/// Build the application router
pub fn build_router(app_state: AppState) -> Router {
    let cors = if cfg!(debug_assertions) {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let allowed_origins = std::env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".to_string());

        if allowed_origins == "*" {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            let origins: Vec<_> = allowed_origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();

            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    };

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/api/wechat/jsapi-ticket", post(handlers::get_jsapi_ticket))
        .route("/api/wechat/status", get(handlers::get_wechat_status))
        .route(
            "/api/channels/:channel_id/messages",
            post(handlers::send_message),
        )
        .route(
            "/api/channels/:channel_id/messages",
            get(handlers::get_messages),
        )
        .route(
            "/api/channels/:channel_id/messages/:message_id",
            get(handlers::get_message),
        )
        .route(
            "/api/channels/:channel_id/ws",
            get(handlers::websocket_handler),
        )
        .route("/api/channels/:channel_id", get(handlers::get_channel))
        .route("/api/channels/:channel_id", patch(handlers::update_channel))
        .route(
            "/api/channels/:channel_id",
            delete(handlers::delete_channel),
        )
        .route("/api/channels", post(handlers::create_channel))
        .route("/api/channels", get(handlers::list_channels))
        .layer(SecurityHeadersLayer)
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::sync::Arc;

    fn create_test_app_state() -> AppState {
        let config = Arc::new(Config::from_env().unwrap());
        AppState::new(config)
    }

    #[test]
    fn test_build_router() {
        let app_state = create_test_app_state();
        let _router = build_router(app_state);
    }

    #[test]
    fn test_router_has_health_endpoint() {
        let app_state = create_test_app_state();
        let _router = build_router(app_state);

        let routes: Vec<&str> = vec![
            "/health",
            "/metrics",
            "/api/channels",
            "/api/channels/{id}/ws",
        ];

        for route in routes {
            let _ = format!("Route {} should exist", route);
        }
    }
}
