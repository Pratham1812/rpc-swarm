use crate::error::Result;
use crate::config::Settings;
use log::{debug, error, info, warn};
use rpc_swarm::config::settings;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Swarm is starting...");
    // Your application logic goes here

    // Create endpoints
    let mut endpoints = Vec::new();
    for endpoint in settings.endpoints {
        let endpoint = Endpoint::new(&endpoint)?;
        endpoints.push(endpoint);
    }

    // Initialize router
    let router = Router::new(endpoints);
    info!("Router initialized with {} endpoints", router.endpoints.len());

    Ok(())
}
