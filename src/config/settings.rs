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
        toml::from_str(&config_str?)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))
    }
}
