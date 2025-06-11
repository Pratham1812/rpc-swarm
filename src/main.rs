use std::collections::HashMap;
use crate::config::Settings;
use crate::error::Result;
use crate::load_balancer::Router;
use log::{debug, error, info, warn};

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
    
    // Load settings first
    let settings = Settings::load()?;
    
    // Store routers for each network
    let mut routers: HashMap<String, Router> = HashMap::new();
    
    // Create endpoints for each network
    for network in networks {
        if let Some(endpoints) = settings.get_network_endpoints(&network) {
            if endpoints.is_empty() {
                warn!("No endpoints found for network: {}", network);
                continue;
            }
            
            info!(
                "Found {} endpoints for network: {}",
                endpoints.len(),
                network
            );
            
            // Initialize router
            let router = Router::new(endpoints.clone());
            info!(
                "Router initialized with {} endpoints for network: {}",
                router.endpoints.len(),
                network
            );
            
            // Store the router
            routers.insert(network.clone(), router);
        } else {
            warn!("Network '{}' not found in configuration", network);
        }
    }
    
    info!("Initialized {} routers", routers.len());
    
    // Your application logic continues here...
    // You can now use the routers HashMap for your swarm logic

    Ok(())
}