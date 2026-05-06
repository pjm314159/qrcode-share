//! Background cleanup task
//!
//! Periodically cleans up expired messages and inactive channels.

use std::time::Duration;

use tokio::time::interval;
use tracing::info;

use crate::state::AppState;

const CLEANUP_INTERVAL_SECS: u64 = 120;
const WECHAT_REFRESH_INTERVAL_SECS: u64 = 7000;
const WECHAT_REFRESH_EARLY_SECS: i64 = 300;

pub fn start_cleanup_task(app_state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

        info!(
            "Cleanup task started (interval: {}s)",
            CLEANUP_INTERVAL_SECS
        );

        loop {
            ticker.tick().await;

            let messages_removed = app_state.cleanup_expired_messages();
            if messages_removed > 0 {
                info!("Cleaned up {} expired messages", messages_removed);
            }

            let channels_removed = app_state.cleanup_inactive_channels();
            if channels_removed > 0 {
                info!("Cleaned up {} inactive channels", channels_removed);
            }

            app_state.message_rate_limiter.cleanup_stale();
            app_state.channel_rate_limiter.cleanup_stale();

            app_state.cleanup_stale_ip_counts();
        }
    })
}

pub fn start_wechat_refresh_task(app_state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(WECHAT_REFRESH_INTERVAL_SECS));

        info!(
            "WeChat token refresh task started (interval: {}s)",
            WECHAT_REFRESH_INTERVAL_SECS
        );

        loop {
            ticker.tick().await;

            let needs_refresh = {
                let cache = app_state.wechat_token_cache.read();
                match cache.as_ref() {
                    Some(c) => {
                        let now = chrono::Utc::now();
                        let access_expires = (c.expires_at - now).num_seconds();
                        let ticket_expires = (c.ticket_expires_at - now).num_seconds();
                        access_expires < WECHAT_REFRESH_EARLY_SECS
                            || ticket_expires < WECHAT_REFRESH_EARLY_SECS
                    }
                    None => true,
                }
            };

            if needs_refresh {
                let is_available = {
                    let status = app_state.wechat_status.read();
                    status.available
                };

                if !is_available {
                    continue;
                }

                if app_state.config.wx_appid.is_none() || app_state.config.wx_secret.is_none() {
                    continue;
                }

                info!("WeChat token approaching expiry, refreshing...");
                app_state.verify_wechat_config().await;
                info!("WeChat token refresh completed");
            }
        }
    })
}

#[allow(dead_code)]
pub fn start_cleanup_task_with_shutdown(
    app_state: AppState,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));

        info!(
            "Cleanup task started (interval: {}s)",
            CLEANUP_INTERVAL_SECS
        );

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let messages_removed = app_state.cleanup_expired_messages();
                    if messages_removed > 0 {
                        info!("Cleaned up {} expired messages", messages_removed);
                    }

                    let channels_removed = app_state.cleanup_inactive_channels();
                    if channels_removed > 0 {
                        info!("Cleaned up {} inactive channels", channels_removed);
                    }

                    app_state.message_rate_limiter.cleanup_stale();
                    app_state.channel_rate_limiter.cleanup_stale();
                    app_state.cleanup_stale_ip_counts();
                }
                _ = shutdown_rx.recv() => {
                    info!("Cleanup task shutting down gracefully");
                    break;
                }
            }
        }
    })
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
    fn test_cleanup_interval_constant() {
        assert_eq!(CLEANUP_INTERVAL_SECS, 120);
    }

    #[test]
    fn test_wechat_refresh_interval_constant() {
        assert_eq!(WECHAT_REFRESH_INTERVAL_SECS, 7000);
    }

    #[test]
    fn test_wechat_refresh_early_secs_constant() {
        assert_eq!(WECHAT_REFRESH_EARLY_SECS, 300);
    }

    #[tokio::test]
    async fn test_start_cleanup_task() {
        let app_state = create_test_app_state();

        // Start the cleanup task
        let handle = start_cleanup_task(app_state);

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // The task should be running
        assert!(!handle.is_finished());

        // Abort the task
        handle.abort();
    }

    #[tokio::test]
    async fn test_start_wechat_refresh_task() {
        let app_state = create_test_app_state();

        let handle = start_wechat_refresh_task(app_state);

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(!handle.is_finished());

        handle.abort();
    }

    #[tokio::test]
    async fn test_cleanup_task_with_shutdown() {
        let app_state = create_test_app_state();
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

        // Start the cleanup task with shutdown support
        let handle = start_cleanup_task_with_shutdown(app_state, shutdown_rx);

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // The task should be running
        assert!(!handle.is_finished());

        // Send shutdown signal
        let _ = shutdown_tx.send(());

        // Wait for task to finish
        tokio::time::sleep(Duration::from_millis(100)).await;

        // The task should have finished
        assert!(handle.is_finished());
    }
}
