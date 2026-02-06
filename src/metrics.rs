use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct NetworkMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
        }
    }

    pub fn increment_total(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_success(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.total_requests.load(Ordering::Relaxed),
            self.successful_requests.load(Ordering::Relaxed),
            self.failed_requests.load(Ordering::Relaxed),
        )
    }
}
