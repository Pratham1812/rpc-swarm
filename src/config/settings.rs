use serde::Deserialize;
use std::fs;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub endpoints: Vec<String>,
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let config_path = "config/default.toml";
        let config_str = fs::read_to_string(config_path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)));
        let settings:Settings =toml::from_str(&config_str?)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)));

        for endpoint in &settings.endpoints {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") && !endpoint.starts_with("ws://") && !endpoint.starts_with("wss://") {
                return Err(Error::Config(format!(
                    "Invalid endpoint URL: {}. Must start with http:// or https://",
                    endpoint
                )));
            }
        }

        
    }
}
