//! Application state management
//!
//! Central state container for all in-memory data.

use std::sync::Arc;

use compact_str::CompactString;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::error::AppError;
use crate::models::CreateChannelRequest;

use super::channel_state::ChannelState;
use super::metrics::Metrics;
use super::rate_limiter::RateLimiter;

type ChannelMap = DashMap<CompactString, Arc<ChannelState>, ahash::RandomState>;
type IpChannelCountMap = DashMap<CompactString, AtomicU64, ahash::RandomState>;

#[derive(Debug, Clone, Default)]
pub struct WechatTokenCache {
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub jsapi_ticket: String,
    pub ticket_expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct WechatStatus {
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    channels: Arc<ChannelMap>,
    ip_channel_counts: Arc<IpChannelCountMap>,
    pub connection_semaphore: Arc<Semaphore>,
    pub message_rate_limiter: Arc<RateLimiter>,
    pub channel_rate_limiter: Arc<RateLimiter>,
    pub metrics: Arc<Metrics>,
    pub wechat_token_cache: Arc<parking_lot::RwLock<Option<WechatTokenCache>>>,
    pub wechat_status: Arc<parking_lot::RwLock<WechatStatus>>,
}

impl AppState {
    pub fn new(config: Arc<Config>) -> Self {
        let wechat_status = if config.wx_appid.is_some() && config.wx_secret.is_some() {
            WechatStatus {
                available: false,
                reason: Some("Pending verification".to_string()),
            }
        } else {
            WechatStatus {
                available: false,
                reason: Some("WX_APPID or WX_SECRET not configured".to_string()),
            }
        };

        Self {
            message_rate_limiter: Arc::new(RateLimiter::with_max_requests(
                config.max_messages_per_minute,
            )),
            channel_rate_limiter: Arc::new(RateLimiter::new(
                config.max_channels_per_user,
                std::time::Duration::from_secs(3600),
            )),
            connection_semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
            channels: Arc::new(DashMap::with_hasher(ahash::RandomState::new())),
            ip_channel_counts: Arc::new(DashMap::with_hasher(ahash::RandomState::new())),
            metrics: Arc::new(Metrics::new()),
            wechat_token_cache: Arc::new(parking_lot::RwLock::new(None)),
            wechat_status: Arc::new(parking_lot::RwLock::new(wechat_status)),
        }
    }

    pub async fn verify_wechat_config(&self) {
        let appid = match self.config.wx_appid.as_ref() {
            Some(id) => id.clone(),
            None => {
                let mut status = self.wechat_status.write();
                *status = WechatStatus {
                    available: false,
                    reason: Some("WX_APPID not configured".to_string()),
                };
                tracing::warn!("WeChat JS-SDK: WX_APPID not configured, WeChat features disabled");
                return;
            }
        };

        let secret = match self.config.wx_secret.as_ref() {
            Some(s) => s.clone(),
            None => {
                let mut status = self.wechat_status.write();
                *status = WechatStatus {
                    available: false,
                    reason: Some("WX_SECRET not configured".to_string()),
                };
                tracing::warn!("WeChat JS-SDK: WX_SECRET not configured, WeChat features disabled");
                return;
            }
        };

        tracing::info!("WeChat JS-SDK: Verifying configuration with APPID={}", appid);

        match Self::fetch_access_token(&appid, &secret).await {
            Ok(token) => {
                tracing::info!("WeChat JS-SDK: access_token obtained successfully");

                match Self::fetch_jsapi_ticket(&token).await {
                    Ok(ticket) => {
                        let now = chrono::Utc::now();
                        let cache = WechatTokenCache {
                            access_token: token,
                            expires_at: now + chrono::Duration::seconds(7000),
                            jsapi_ticket: ticket,
                            ticket_expires_at: now + chrono::Duration::seconds(7000),
                        };
                        {
                            let mut guard = self.wechat_token_cache.write();
                            *guard = Some(cache);
                        }
                        {
                            let mut status = self.wechat_status.write();
                            *status = WechatStatus {
                                available: true,
                                reason: None,
                            };
                        }
                        tracing::info!("WeChat JS-SDK: Configuration verified, jsapi_ticket obtained, WeChat features enabled");
                    }
                    Err(e) => {
                        let reason = format!("jsapi_ticket failed: {}", e);
                        let mut status = self.wechat_status.write();
                        *status = WechatStatus {
                            available: false,
                            reason: Some(reason.clone()),
                        };
                        tracing::error!("WeChat JS-SDK: {} - Please check if your server IP is whitelisted in WeChat Official Account settings and JS-SDK safe domain is configured", reason);
                    }
                }
            }
            Err(e) => {
                let reason = format!("access_token failed: {}", e);
                let mut status = self.wechat_status.write();
                *status = WechatStatus {
                    available: false,
                    reason: Some(reason.clone()),
                };
                tracing::error!("WeChat JS-SDK: {} - Please check WX_APPID and WX_SECRET are correct, and your server IP is whitelisted in WeChat Official Account settings (Settings > Basic > IP Whitelist)", reason);
            }
        }
    }

    async fn fetch_access_token(appid: &str, secret: &str) -> Result<String, String> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            appid, secret
        );

        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse response failed: {}", e))?;

        if let Some(token) = data.get("access_token").and_then(|v| v.as_str()) {
            Ok(token.to_string())
        } else {
            let errcode = data.get("errcode").and_then(|v| v.as_i64()).unwrap_or(0);
            let errmsg = data
                .get("errmsg")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            Err(format!("errcode={}, errmsg={}", errcode, errmsg))
        }
    }

    async fn fetch_jsapi_ticket(access_token: &str) -> Result<String, String> {
        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/ticket/getticket?access_token={}&type=jsapi",
            access_token
        );

        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse response failed: {}", e))?;

        let errcode = data.get("errcode").and_then(|v| v.as_i64()).unwrap_or(-1);
        if errcode != 0 {
            let errmsg = data
                .get("errmsg")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("errcode={}, errmsg={}", errcode, errmsg));
        }

        data.get("ticket")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Missing ticket in response".to_string())
    }

    pub fn create_channel(
        &self,
        request: CreateChannelRequest,
        creator_ip: &str,
    ) -> Result<(String, Arc<ChannelState>), AppError> {
        if self.channels.len() >= self.config.max_channels {
            return Err(AppError::ChannelLimitReached);
        }

        let user_channels = self.count_user_channels(creator_ip);
        if user_channels >= self.config.max_channels_per_user {
            return Err(AppError::UserChannelLimit);
        }

        request.validate().map_err(AppError::ValidationError)?;

        let channel_id = generate_channel_id();

        let mut channel_state = ChannelState::new(channel_id.clone(), request.name);
        channel_state.creator_ip = Some(CompactString::from(creator_ip));

        if let Some(password) = request.password {
            let hash = crate::auth::hash_password(&password)
                .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
            channel_state = channel_state.with_password(Some(hash));
        }

        if let Some(limitation) = request.link_limitation {
            channel_state = channel_state.with_link_limitation(limitation);
        }

        let channel_state = Arc::new(channel_state);

        self.channels
            .insert(channel_id.clone().into(), channel_state.clone());

        self.increment_ip_channel_count(creator_ip);

        self.metrics.inc_channels();

        Ok((channel_id, channel_state))
    }

    pub fn get_channel(&self, id: &str) -> Option<Arc<ChannelState>> {
        self.channels.get(id).map(|entry| entry.value().clone())
    }

    pub fn delete_channel(&self, id: &str) -> bool {
        if let Some((_, channel)) = self.channels.remove(id) {
            if let Some(ref ip) = channel.creator_ip {
                self.decrement_ip_channel_count(ip);
            }
            self.metrics.dec_channels();
            true
        } else {
            false
        }
    }

    pub fn list_channels(&self, page: usize, limit: usize) -> Vec<Arc<ChannelState>> {
        let skip = page.saturating_sub(1) * limit;

        self.channels
            .iter()
            .skip(skip)
            .take(limit)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    fn count_user_channels(&self, ip: &str) -> usize {
        self.ip_channel_counts
            .get(ip)
            .map(|g| g.load(Ordering::Relaxed) as usize)
            .unwrap_or(0)
    }

    fn increment_ip_channel_count(&self, ip: &str) {
        self.ip_channel_counts
            .entry(CompactString::from(ip))
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_ip_channel_count(&self, ip: &str) {
        if let Some(counter) = self.ip_channel_counts.get(ip) {
            let prev = counter.fetch_sub(1, Ordering::Relaxed);
            if prev <= 1 {
                drop(counter);
                self.ip_channel_counts.remove_if(ip, |_, v| v.load(Ordering::Relaxed) == 0);
            }
        }
    }

    pub fn cleanup_expired_messages(&self) -> usize {
        let mut total_removed = 0;

        for entry in self.channels.iter() {
            let channel = entry.value();
            let removed = channel.remove_expired();
            if removed > 0 {
                total_removed += removed;
                for _ in 0..removed {
                    self.metrics.dec_messages();
                }
            }
        }

        total_removed
    }

    pub fn cleanup_inactive_channels(&self) -> usize {
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.config.channel_ttl.as_secs() as i64);

        let mut removed = 0;

        self.channels.retain(|_, channel| {
            let should_keep = channel.last_activity > cutoff;
            if !should_keep {
                removed += 1;
                self.metrics.dec_channels();
                if let Some(ref ip) = channel.creator_ip {
                    self.decrement_ip_channel_count(ip);
                }
            }
            should_keep
        });

        removed
    }

    pub fn cleanup_stale_ip_counts(&self) {
        self.ip_channel_counts.retain(|_, count| {
            count.load(Ordering::Relaxed) > 0
        });
    }
}

fn generate_channel_id() -> String {
    uuid::Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("abcdefgh")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Message;

    fn create_test_config() -> Arc<Config> {
        Arc::new(Config::from_env().unwrap())
    }

    fn create_test_request(name: &str) -> CreateChannelRequest {
        CreateChannelRequest {
            name: name.to_string(),
            password: None,
            link_limitation: None,
            channel_type: None,
            location: None,
            teacher: None,
        }
    }

    #[test]
    fn test_app_state_new() {
        let config = create_test_config();
        let state = AppState::new(config);

        assert_eq!(state.channel_count(), 0);
    }

    #[test]
    fn test_app_state_clone() {
        let config = create_test_config();
        let state = AppState::new(config);
        let cloned = state.clone();

        assert_eq!(state.channel_count(), cloned.channel_count());
    }

    #[test]
    fn test_create_channel() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request = create_test_request("Test Channel");
        let result = state.create_channel(request, "127.0.0.1");

        assert!(result.is_ok());
        let (id, channel) = result.unwrap();
        assert_eq!(id.len(), 8);
        assert_eq!(channel.name, "Test Channel");
        assert_eq!(state.channel_count(), 1);
    }

    #[test]
    fn test_get_channel() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request = create_test_request("Test");
        let (id, _) = state.create_channel(request, "127.0.0.1").unwrap();

        let found = state.get_channel(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test");
    }

    #[test]
    fn test_delete_channel() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request = create_test_request("Test");
        let (id, _) = state.create_channel(request, "127.0.0.1").unwrap();

        assert!(state.delete_channel(&id));
        assert_eq!(state.channel_count(), 0);
        assert!(!state.delete_channel(&id));
    }

    #[test]
    fn test_list_channels() {
        let config = create_test_config();
        let state = AppState::new(config);

        for i in 0..5 {
            let request = create_test_request(&format!("Channel {}", i));
            state.create_channel(request, "127.0.0.1").unwrap();
        }

        let page1 = state.list_channels(1, 3);
        assert_eq!(page1.len(), 3);

        let page2 = state.list_channels(2, 3);
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_channel_limit() {
        let mut config = Config::from_env().unwrap();
        config.max_channels = 2;
        let config = Arc::new(config);
        let state = AppState::new(config);

        let request1 = create_test_request("Channel 1");
        let request2 = create_test_request("Channel 2");
        let request3 = create_test_request("Channel 3");

        assert!(state.create_channel(request1, "127.0.0.1").is_ok());
        assert!(state.create_channel(request2, "127.0.0.1").is_ok());

        assert!(matches!(
            state.create_channel(request3, "127.0.0.1"),
            Err(AppError::ChannelLimitReached)
        ));
    }

    #[test]
    fn test_cleanup_expired_messages() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request = create_test_request("Test");
        let (_, channel) = state.create_channel(request, "127.0.0.1").unwrap();

        let msg = Arc::new(Message::new(
            "msg1",
            "Test",
            "https://example.com",
            std::time::Duration::from_secs(3600),
        ));
        channel.add_message(msg);

        assert_eq!(channel.message_count(), 1);

        let removed = state.cleanup_expired_messages();
        assert_eq!(removed, 0);
        assert_eq!(channel.message_count(), 1);
    }

    #[test]
    fn test_cleanup_inactive_channels() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request1 = create_test_request("Active Channel");
        let request2 = create_test_request("Inactive Channel");

        state.create_channel(request1, "127.0.0.1").unwrap();
        state.create_channel(request2, "127.0.0.1").unwrap();

        assert_eq!(state.channel_count(), 2);

        let removed = state.cleanup_inactive_channels();
        assert_eq!(removed, 0);
        assert_eq!(state.channel_count(), 2);
    }

    #[test]
    fn test_per_ip_channel_counting() {
        let config = create_test_config();
        let state = AppState::new(config);

        let r1 = create_test_request("Channel 1");
        let r2 = create_test_request("Channel 2");
        let r3 = create_test_request("Channel 3");

        state.create_channel(r1, "192.168.1.1").unwrap();
        state.create_channel(r2, "192.168.1.1").unwrap();
        state.create_channel(r3, "192.168.1.2").unwrap();

        assert_eq!(state.count_user_channels("192.168.1.1"), 2);
        assert_eq!(state.count_user_channels("192.168.1.2"), 1);
        assert_eq!(state.count_user_channels("10.0.0.1"), 0);
    }

    #[test]
    fn test_per_ip_channel_counting_on_delete() {
        let config = create_test_config();
        let state = AppState::new(config);

        let r1 = create_test_request("Channel 1");
        let r2 = create_test_request("Channel 2");

        let (id1, _) = state.create_channel(r1, "192.168.1.1").unwrap();
        let (id2, _) = state.create_channel(r2, "192.168.1.1").unwrap();

        assert_eq!(state.count_user_channels("192.168.1.1"), 2);

        state.delete_channel(&id1);
        assert_eq!(state.count_user_channels("192.168.1.1"), 1);

        state.delete_channel(&id2);
        assert_eq!(state.count_user_channels("192.168.1.1"), 0);
    }

    #[test]
    fn test_user_channel_limit_per_ip() {
        let mut config = Config::from_env().unwrap();
        config.max_channels_per_user = 2;
        let config = Arc::new(config);
        let state = AppState::new(config);

        let r1 = create_test_request("Channel 1");
        let r2 = create_test_request("Channel 2");
        let r3 = create_test_request("Channel 3");

        assert!(state.create_channel(r1, "192.168.1.1").is_ok());
        assert!(state.create_channel(r2, "192.168.1.1").is_ok());

        assert!(matches!(
            state.create_channel(r3, "192.168.1.1"),
            Err(AppError::UserChannelLimit)
        ));

        let r4 = create_test_request("Channel 4");
        assert!(state.create_channel(r4, "192.168.1.2").is_ok());
    }

    #[test]
    fn test_creator_ip_stored_in_channel() {
        let config = create_test_config();
        let state = AppState::new(config);

        let request = create_test_request("Test");
        let (_, channel) = state.create_channel(request, "10.0.0.1").unwrap();

        assert_eq!(channel.creator_ip.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn test_connection_semaphore() {
        let config = create_test_config();
        let max_connections = config.max_connections;
        let state = AppState::new(config);

        assert_eq!(state.connection_semaphore.available_permits(), max_connections);
    }

    #[test]
    fn test_cleanup_stale_ip_counts() {
        let config = create_test_config();
        let state = AppState::new(config);

        let r1 = create_test_request("Channel 1");
        let (id1, _) = state.create_channel(r1, "192.168.1.1").unwrap();

        assert_eq!(state.count_user_channels("192.168.1.1"), 1);

        state.delete_channel(&id1);
        assert_eq!(state.count_user_channels("192.168.1.1"), 0);

        state.cleanup_stale_ip_counts();
        assert_eq!(state.count_user_channels("192.168.1.1"), 0);
    }

    #[test]
    fn test_cleanup_inactive_channels_decrements_ip_count() {
        let config = create_test_config();
        let state = AppState::new(config);

        let r1 = create_test_request("Channel 1");
        state.create_channel(r1, "10.0.0.5").unwrap();

        assert_eq!(state.count_user_channels("10.0.0.5"), 1);

        let removed = state.cleanup_inactive_channels();
        assert_eq!(removed, 0);
        assert_eq!(state.count_user_channels("10.0.0.5"), 1);
    }
}
