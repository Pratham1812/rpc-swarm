use crate::state::NetworkState;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

pub async fn spawn_health_checker(
    network_name: String,
    network_state: Arc<NetworkState>,
    interval_secs: u64,
) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval_secs);
        let mut interval_timer = tokio::time::interval(interval);

        tracing::info!(
            network = %network_name,
            interval_secs = interval_secs,
            "Starting health checker"
        );

        loop {
            interval_timer.tick().await;

            for endpoint in &network_state.endpoints {
                check_endpoint(endpoint, &network_state).await;
            }
        }
    });
}

async fn check_endpoint(endpoint: &str, network_state: &NetworkState) {
    let result = perform_health_check(endpoint).await;

    let mut health_status = network_state.health_status.write().await;

    if let Some(status) = health_status.get_mut(endpoint) {
        match result {
            Ok(_) => {
                let was_unhealthy = !status.is_healthy;
                status.mark_healthy();
                if was_unhealthy {
                    tracing::info!(endpoint = %endpoint, "Endpoint is now healthy");
                }
            }
            Err(e) => {
                let was_healthy = status.is_healthy;
                status.mark_unhealthy();

                if was_healthy && !status.is_healthy {
                    tracing::warn!(
                        endpoint = %endpoint,
                        error = %e,
                        "Endpoint marked unhealthy"
                    );
                } else {
                    tracing::debug!(
                        endpoint = %endpoint,
                        error = %e,
                        failures = status.consecutive_failures,
                        "Health check failed"
                    );
                }
            }
        }
    }
}

async fn perform_health_check(endpoint: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let response = client
        .post(endpoint)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP status: {}", response.status()));
    }

    let response_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Check if response contains "result" field (not "error")
    if response_body.get("result").is_some() {
        Ok(())
    } else if let Some(error) = response_body.get("error") {
        Err(format!("JSON-RPC error: {}", error))
    } else {
        Err("Invalid JSON-RPC response".to_string())
    }
}
