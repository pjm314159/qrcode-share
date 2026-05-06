//! QRcode Share Backend Library
//!
//! This library provides the core functionality for the QRcode Share application.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod router;
pub mod state;
pub mod tasks;

pub use auth::{check_channel_access, hash_password, verify_password};
pub use config::Config;
pub use db::{ChannelRepository, Database};
pub use error::{ApiResponse, AppError, ErrorCode};
pub use middleware::{RateLimitLayer, SecurityHeadersLayer};
pub use router::build_router;
pub use state::{AppState, ChannelState, Metrics, RateLimiter};
pub use tasks::{start_cleanup_task, start_wechat_refresh_task};
