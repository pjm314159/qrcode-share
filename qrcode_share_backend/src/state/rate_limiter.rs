//! Rate limiting implementation
//!
//! Provides adaptive rate limiting with automatic cleanup of stale entries.

use std::time::{Duration, Instant};

use compact_str::CompactString;
use dashmap::DashMap;
use smallvec::SmallVec;

/// Default rate limit window (1 minute)
const DEFAULT_WINDOW_SECS: u64 = 60;

/// Number of timestamps to inline in SmallVec
const INLINE_TIMESTAMPS: usize = 8;

/// Adaptive rate limiter using DashMap for concurrent access
#[derive(Debug)]
pub struct RateLimiter {
    /// Request timestamps per key
    requests: DashMap<CompactString, SmallVec<[Instant; INLINE_TIMESTAMPS]>>,
    /// Maximum requests per window
    max_requests: usize,
    /// Time window duration
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: DashMap::new(),
            max_requests,
            window,
        }
    }

    /// Create with default window (1 minute)
    pub fn with_max_requests(max_requests: usize) -> Self {
        Self::new(max_requests, Duration::from_secs(DEFAULT_WINDOW_SECS))
    }

    /// Check if a request is allowed for the given key
    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;

        // Get or create entry
        let mut entry = self.requests.entry(CompactString::from(key)).or_default();

        // Remove old timestamps (amortized cleanup)
        entry.retain(|timestamp| *timestamp > cutoff);

        // Check if under limit
        if entry.len() >= self.max_requests {
            return false;
        }

        // Record this request
        entry.push(now);
        true
    }

    /// Get remaining requests for a key
    pub fn remaining(&self, key: &str) -> usize {
        let now = Instant::now();
        let cutoff = now - self.window;

        if let Some(entry) = self.requests.get(key) {
            let count = entry.iter().filter(|&&ts| ts > cutoff).count();
            self.max_requests.saturating_sub(count)
        } else {
            self.max_requests
        }
    }

    /// Get the time until the rate limit resets for a key
    pub fn reset_after(&self, key: &str) -> Option<Duration> {
        let now = Instant::now();
        let cutoff = now - self.window;

        if let Some(entry) = self.requests.get(key) {
            // Find the oldest timestamp within the window
            let oldest = entry.iter().filter(|&&ts| ts > cutoff).min().copied();

            if let Some(ts) = oldest {
                let reset_time = ts + self.window;
                if reset_time > now {
                    return Some(reset_time - now);
                }
            }
        }

        None
    }

    /// Clean up stale entries (call periodically)
    pub fn cleanup_stale(&self) {
        let now = Instant::now();
        let cutoff = now - self.window - Duration::from_secs(60); // Extra buffer

        self.requests.retain(|_, timestamps| {
            timestamps.retain(|ts| *ts > cutoff);
            !timestamps.is_empty()
        });
    }

    /// Get total number of tracked keys
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        for _ in 0..5 {
            assert!(limiter.check("user1"));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));

        assert!(limiter.check("user1"));
        assert!(limiter.check("user1"));
        assert!(limiter.check("user1"));
        assert!(!limiter.check("user1")); // Should be blocked
    }

    #[test]
    fn test_rate_limiter_independent_keys() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        assert!(limiter.check("user1"));
        assert!(limiter.check("user1"));
        assert!(!limiter.check("user1"));

        // Different key should have separate limit
        assert!(limiter.check("user2"));
        assert!(limiter.check("user2"));
        assert!(!limiter.check("user2"));
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        assert_eq!(limiter.remaining("user1"), 5);

        limiter.check("user1");
        assert_eq!(limiter.remaining("user1"), 4);

        limiter.check("user1");
        limiter.check("user1");
        assert_eq!(limiter.remaining("user1"), 2);
    }

    #[test]
    fn test_rate_limiter_cleanup() {
        let limiter = RateLimiter::new(1, Duration::from_millis(10));

        // Add a request
        assert!(limiter.check("user1"));

        // Wait for window to expire
        std::thread::sleep(Duration::from_millis(20));

        // Cleanup should remove stale entries
        limiter.cleanup_stale();

        // Should be able to make a new request
        assert!(limiter.check("user1"));
    }
}
