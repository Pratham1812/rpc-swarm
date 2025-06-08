use crate::load_balancer::endpoint::Endpoint;
use crate::health::checker::HealthChecker;
use crate::error::Result;

pub struct HealthMonitor {
    health_checker: HealthChecker,
}

impl HealthMonitor{
    pub fn new(health_cheker:HealthChecker) -> Result<Self> {
        Ok(HealthMonitor {
            health_checker:HealthChecker
        })
    }

    pub async fn monitor_health(&self,endpoints: &mut [Endpoint]) -> Result<()>{
        for endpoint in endpoints.iter_mut() {
            if let Err(e) = self.health_checker.check_health(endpoint).await {
                endpoint.set_healthy(false);
                log::error!("Health check failed for endpoint {}: {}", endpoint.url, e);
            } else {
                endpoint.set_healthy(true);
                log::info!("Endpoint {} is healthy", endpoint.url);
            }
        }
        Ok(())
    }
}