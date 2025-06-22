use tokio::time::{interval, Duration};
use log::{info, error};

use crate::health::checker::HealthChecker;
use crate::load_balancer::endpoint::Endpoint;
use crate::error::Result;

pub struct HealthMonitor {
    checker: HealthChecker,
    interval: Duration,
}

impl HealthMonitor {
    pub fn new(interval_secs: u64, timeout_secs: u64) -> Self {
        HealthMonitor {
            checker: HealthChecker::new(timeout_secs),
            interval: Duration::from_secs(interval_secs),
        }
    }

    // Start monitoring endpoints
    pub async fn monitor(&self, endpoints: &mut [Endpoint]) -> Result<()> {
        let mut ticker = interval(self.interval);
        loop {
            ticker.tick().await;
            info!("Starting health check cycle for {} endpoints", endpoints.len());
            for endpoint in endpoints.iter_mut() {
                match self.checker.check_health(endpoint).await {    // Create a new health monitor

                    Ok(()) => info!("Endpoint {} is healthy", endpoint.url),
                    Err(e) => {
                        error!("Endpoint {} health check failed: {}", endpoint.url, e);
                        endpoint.set_healthy(false);
                    }
                }
            }
        }
    }
}