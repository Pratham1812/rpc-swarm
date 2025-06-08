use crate::error::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use url::Url;

#[derive(Debug)]
pub struct Endpoint {
    pub url: Url,
    pub network: String,
    active_connections: AtomicUsize,
    healthy: bool,
}
// The endpoint struct represents the composition of a rpc endpoint
// It contains the rpc_url and network name as the publically accessible variables
impl Endpoint {
    pub fn new(url: Url, network: String) -> Result<Self> {
        Ok(Endpoint {
            url,
            network,
            active_connections: AtomicUsize::new(0),
            healthy: true,
        })
    }
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }
    pub fn get_connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }
    pub fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}
