use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub health_check: HealthCheckConfig,
    pub networks: HashMap<String, NetworkConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheckConfig {
    pub interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub endpoints: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("RPC"))
            .build()
            .context("Failed to build configuration")?;

        let config = settings
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.networks.is_empty() {
            anyhow::bail!("At least one network must be configured");
        }

        for (name, network) in &self.networks {
            if network.endpoints.is_empty() {
                anyhow::bail!("Network '{}' has no endpoints configured", name);
            }

            for endpoint in &network.endpoints {
                if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                    anyhow::bail!(
                        "Invalid endpoint '{}' in network '{}': must start with http:// or https://",
                        endpoint,
                        name
                    );
                }
            }
        }

        if self.health_check.interval_secs == 0 {
            anyhow::bail!("Health check interval must be greater than 0");
        }

        Ok(())
    }
}
