//! Database layer
//!
//! This module provides database connection and repository implementations.

mod channel_repo;
mod database;

pub use channel_repo::ChannelRepository;
pub use database::Database;
