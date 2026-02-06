use crate::error::ProxyError;
use crate::state::NetworkState;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub async fn select_endpoint(
    network_name: &str,
    network_state: &Arc<NetworkState>,
) -> Result<String, ProxyError> {
    let health_status = network_state.health_status.read().await;

    // Get all healthy endpoints
    let healthy_endpoints: Vec<&String> = network_state
        .endpoints
        .iter()
        .filter(|endpoint| {
            health_status
                .get(*endpoint)
                .map(|status| status.is_healthy)
                .unwrap_or(false)
        })
        .collect();

    if healthy_endpoints.is_empty() {
        return Err(ProxyError::NoHealthyEndpoints(network_name.to_string()));
    }

    // Round-robin selection
    let counter = network_state
        .round_robin_counter
        .fetch_add(1, Ordering::Relaxed);
    let index = counter % healthy_endpoints.len();

    let selected = healthy_endpoints[index].clone();

    tracing::debug!(
        network = %network_name,
        endpoint = %selected,
        healthy_count = healthy_endpoints.len(),
        total_count = network_state.endpoints.len(),
        "Selected endpoint"
    );

    Ok(selected)
}
