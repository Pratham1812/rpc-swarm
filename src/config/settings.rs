use std::collections::HashMap;
use std::env;

use crate::{
    error::{Error, Result},
    load_balancer::Endpoint,
};
use dotenvy::dotenv;
use serde::Deserialize;
use std::sync::atomic::AtomicUsize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub network_endpoints: HashMap<String, Vec<Endpoint>>,
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let networks: Vec<String> = vec![
            "BASE".to_string(),
            "ARBITRUM".to_string(),
            "MAINNET".to_string(),
        ];

        // Load .env file
        dotenv().map_err(|e| Error::Config(format!("Failed to load .env: {}", e)))?;

        let mut network_endpoints: HashMap<String, Vec<Endpoint>> = HashMap::new();

        for network in &networks {
            let env_var = format!("{}_RPC_ENDPOINTS", network.to_uppercase());
            let network_endpoints_str = std::env::var(&env_var)
                .map_err(|e| Error::Config(format!("Missing {} in .env: {}", env_var, e)))?;

            // Split comma-separated endpoints and add to vector
            let endpoints_str: Vec<String> = network_endpoints_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut endpoints: Vec<Endpoint> = Vec::new();

            // Validate endpoints for this network
            for endpoint in &endpoints_str {
                let url = Url::parse(endpoint)
                    .map_err(|e| Error::Config(format!("Invalid URL '{}': {}", endpoint, e)))?;

                if !url.scheme().starts_with("http") && !url.scheme().starts_with("ws") {
                    return Err(Error::Config(format!("Invalid URL scheme: {}", endpoint)));
                }
                endpoints.push(Endpoint {
                    url,
                    network: network.clone(),
                    active_connections: AtomicUsize::new(0),
                    healthy: true,
                });
            }

            // Add to the network-specific HashMap
            network_endpoints.insert(network.clone(), endpoints);
        }

        // Default health check settings
        Ok(Settings {
            network_endpoints,
            health_check_interval_secs: 10,
            health_check_timeout_secs: 2,
        })
    }

    // Helper method to get endpoints for a specific network
    pub fn get_network_endpoints(&self, network: &str) -> Option<&Vec<Endpoint>> {
        self.network_endpoints.get(network)
    }

    // Helper method to get all endpoints as a flat vector
    // Helper method to get all endpoints as a flat vector
    pub fn get_all_endpoints(&self) -> Vec<Endpoint> {
        self.network_endpoints
            .values()
            .flat_map(|endpoints| endpoints.iter())
            .cloned() // Clone each endpoint
            .collect()
    }

    // Helper method to get all network names
    pub fn get_networks(&self) -> Vec<&String> {
        self.network_endpoints.keys().collect()
    }
}
