//! Application state management
//!
//! This module provides the in-memory state management for the application,
//! including channels, messages, rate limiting, and metrics.

mod app_state;
mod channel_state;
mod metrics;
mod rate_limiter;

pub use app_state::{AppState, WechatTokenCache, WechatStatus};
pub use channel_state::{ChannelEvent, ChannelState};
pub use metrics::Metrics;
pub use rate_limiter::RateLimiter;
