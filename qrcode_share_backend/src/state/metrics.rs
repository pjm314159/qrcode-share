use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[repr(align(64))]
#[derive(Debug)]
struct CacheLine<T> {
    value: T,
}

impl<T> CacheLine<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct Metrics {
    messages_sent: CacheLine<AtomicU64>,
    messages_received: CacheLine<AtomicU64>,
    active_connections: CacheLine<AtomicUsize>,
    active_channels: CacheLine<AtomicUsize>,
    total_channels: CacheLine<AtomicU64>,
    total_messages: CacheLine<AtomicU64>,
    total_subscribers: CacheLine<AtomicUsize>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            messages_sent: CacheLine::new(AtomicU64::new(0)),
            messages_received: CacheLine::new(AtomicU64::new(0)),
            active_connections: CacheLine::new(AtomicUsize::new(0)),
            active_channels: CacheLine::new(AtomicUsize::new(0)),
            total_channels: CacheLine::new(AtomicU64::new(0)),
            total_messages: CacheLine::new(AtomicU64::new(0)),
            total_subscribers: CacheLine::new(AtomicUsize::new(0)),
        }
    }

    pub fn inc_messages_sent(&self, count: u64) {
        self.messages_sent.value.fetch_add(count, Ordering::Relaxed);
    }

    pub fn inc_messages(&self) {
        self.messages_sent.value.fetch_add(1, Ordering::Relaxed);
        self.total_messages.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_messages(&self) {
        self.total_messages.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn inc_messages_received(&self, count: u64) {
        self.messages_received.value.fetch_add(count, Ordering::Relaxed);
    }

    pub fn inc_connections(&self) {
        self.active_connections.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_connections(&self) {
        self.active_connections.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn inc_channels(&self) {
        self.active_channels.value.fetch_add(1, Ordering::Relaxed);
        self.total_channels.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_channels(&self) {
        self.active_channels.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.value.load(Ordering::Relaxed)
    }

    pub fn message_count(&self) -> u64 {
        self.messages_sent.value.load(Ordering::Relaxed)
    }

    pub fn messages_received(&self) -> u64 {
        self.messages_received.value.load(Ordering::Relaxed)
    }

    pub fn active_connections(&self) -> usize {
        self.active_connections.value.load(Ordering::Relaxed)
    }

    pub fn active_channels(&self) -> usize {
        self.active_channels.value.load(Ordering::Relaxed)
    }

    pub fn channel_count(&self) -> u64 {
        self.total_channels.value.load(Ordering::Relaxed)
    }

    pub fn total_message_count(&self) -> u64 {
        self.total_messages.value.load(Ordering::Relaxed)
    }

    pub fn inc_subscribers(&self) {
        self.total_subscribers.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_subscribers(&self) {
        self.total_subscribers.value.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn total_subscribers(&self) -> usize {
        self.total_subscribers.value.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_increment() {
        let metrics = Metrics::new();

        metrics.inc_messages_sent(5);
        assert_eq!(metrics.messages_sent(), 5);

        metrics.inc_messages_received(3);
        assert_eq!(metrics.messages_received(), 3);
    }

    #[test]
    fn test_metrics_connections() {
        let metrics = Metrics::new();

        metrics.inc_connections();
        metrics.inc_connections();
        assert_eq!(metrics.active_connections(), 2);

        metrics.dec_connections();
        assert_eq!(metrics.active_connections(), 1);
    }

    #[test]
    fn test_metrics_channels() {
        let metrics = Metrics::new();

        metrics.inc_channels();
        assert_eq!(metrics.active_channels(), 1);

        metrics.dec_channels();
        assert_eq!(metrics.active_channels(), 0);
    }

    #[test]
    fn test_metrics_inc_messages() {
        let metrics = Metrics::new();

        metrics.inc_messages();
        metrics.inc_messages();
        assert_eq!(metrics.message_count(), 2);
        assert_eq!(metrics.total_message_count(), 2);
    }

    #[test]
    fn test_metrics_dec_messages() {
        let metrics = Metrics::new();

        metrics.inc_messages();
        metrics.inc_messages();
        metrics.dec_messages();
        assert_eq!(metrics.total_message_count(), 1);
    }

    #[test]
    fn test_cache_line_alignment() {
        assert_eq!(std::mem::size_of::<CacheLine<AtomicU64>>(), 64);
    }

    #[test]
    fn test_metrics_subscribers() {
        let metrics = Metrics::new();

        metrics.inc_subscribers();
        metrics.inc_subscribers();
        metrics.inc_subscribers();
        assert_eq!(metrics.total_subscribers(), 3);

        metrics.dec_subscribers();
        assert_eq!(metrics.total_subscribers(), 2);
    }
}
