use std::collections::HashMap;
use rpc_swarm::config::Settings;
use rpc_swarm::error::Result;
use rpc_swarm::load_balancer::Router;
use rpc_swarm::health::monitor::HealthMonitor;
use log::{debug, error, info, warn};
use tokio::task;
use tokio::time::Duration;
// You'll need to import your Router struct
// use crate::router::Router;  // Adjust path as needed

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let networks: Vec<String> = vec![
        "BASE".to_string(),
        "ARBITRUM".to_string(),
        "MAINNET".to_string(),
    ];
    
    info!("Swarm is starting...");
    
    let settings = Settings::load()?;
    let mut routers: HashMap<String, Router> = HashMap::new();

    let monitor = HealthMonitor::new(
        settings.health_check_interval_secs,
        settings.health_check_timeout_secs,
    );
    
    for network in networks {
        if let Some(endpoints) = settings.get_network_endpoints(&network) {
            if endpoints.is_empty() {
                warn!("No endpoints found for network: {}", network);
                continue;
            }
            
            info!("Found {} endpoints for network: {}", endpoints.len(), network);
            
            let router = Router::new(network.clone(), endpoints.clone())?;
            info!("Router initialized with {} endpoints for network: {}", endpoints.len(), network);

            // Clone data for the health monitor task
            let mut connections_for_monitor = router.connections.clone();
            let network_name = network.clone();
            
            task::spawn(async move {
                if let Err(e) = monitor.monitor(&mut connections_for_monitor).await {
                    error!("Health monitor failed for {}: {}", network_name, e);
                }
            }); 

            routers.insert(network.clone(), router);
        } else {
            warn!("Network '{}' not found in configuration", network);
        }
    }
    
    info!("Initialized {} routers", routers.len());
    
    // Keep running
    tokio::time::sleep(Duration::from_secs(3600)).await;

    Ok(())
}