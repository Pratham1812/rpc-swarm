use crate::metrics::NetworkMetrics;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;

const MAX_CONSECUTIVE_FAILURES: u32 = 3;

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: Instant,
    pub consecutive_failures: u32,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true, // Start optimistic
            last_check: Instant::now(),
            consecutive_failures: 0,
        }
    }
}

impl HealthStatus {
    pub fn mark_healthy(&mut self) {
        if !self.is_healthy {
            tracing::warn!("Endpoint recovered and is now healthy");
        }
        self.is_healthy = true;
        self.consecutive_failures = 0;
        self.last_check = Instant::now();
    }

    pub fn mark_unhealthy(&mut self) {
        self.consecutive_failures += 1;
        self.last_check = Instant::now();

        if self.consecutive_failures >= MAX_CONSECUTIVE_FAILURES && self.is_healthy {
            self.is_healthy = false;
            tracing::warn!(
                "Endpoint marked unhealthy after {} consecutive failures",
                MAX_CONSECUTIVE_FAILURES
            );
        }
    }
}

#[derive(Debug)]
pub struct NetworkState {
    pub endpoints: Vec<String>,
    pub health_status: Arc<RwLock<HashMap<String, HealthStatus>>>,
    pub round_robin_counter: AtomicUsize,
    pub metrics: Arc<NetworkMetrics>,
}

impl NetworkState {
    pub fn new(endpoints: Vec<String>) -> Self {
        let mut health_status = HashMap::new();
        for endpoint in &endpoints {
            health_status.insert(endpoint.clone(), HealthStatus::default());
        }

        Self {
            endpoints,
            health_status: Arc::new(RwLock::new(health_status)),
            round_robin_counter: AtomicUsize::new(0),
            metrics: Arc::new(NetworkMetrics::new()),
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub networks: HashMap<String, Arc<NetworkState>>,
}

impl AppState {
    pub fn new(config: &crate::config::Config) -> Self {
        let mut networks = HashMap::new();

        for (name, network_config) in &config.networks {
            let network_state = Arc::new(NetworkState::new(network_config.endpoints.clone()));
            networks.insert(name.clone(), network_state);
        }

        Self { networks }
    }

    pub fn get_network(&self, name: &str) -> Option<Arc<NetworkState>> {
        self.networks.get(name).cloned()
    }
}
