use hyper::{Client, Request, Body};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::{json, Value};
use url::Url;
use std::time::Duration;

use crate::load_balancer::endpoint::Endpoint;
use  crate::error::{Error, Result};

pub struct HealthChecker{
    http_client: Client<hyper::client::HttpConnector>,
    timeout: Duration,
}

impl HealthChecker{

    pub fn new(timeout_secs: u64) -> Self {
        let http_client = Client::new();
        let timeout = Duration::from_secs(timeout_secs);
        HealthChecker { http_client, timeout }
    }

    pub async fn check_health(&self, endpoint: &mut Endpoint) -> Result<>{
        match endpoint.url.scheme() {
            "http" | "https" => self.check_http_health(endpoint).await,
            "ws" | "wss" => self.check_ws_health(endpoint).await,
            _ => Err(Error::HealthCheck(format!("Unsupported URL scheme: {}", endpoint.url))),
        }
    }

    // HTTP health check: Send eth_blockNumber JSON-RPC request
    async fn check_http(&self, endpoint: &mut Endpoint) -> Result<()> {
        // Build JSON-RPC request
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });
        let req = Request::builder()
            .method("POST")
            .uri(endpoint.url.as_str())
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))?;

        // Send request with timeout
        let response = tokio::time::timeout(self.timeout, self.http_client.request(req)).await??;
        let status = response.status();
        if !status.is_success() {
            endpoint.set_healthy(false);
            return Err(Error::Config(format!("HTTP check failed with status: {}", status)));
        }

        // Parse response
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        let json: Value = serde_json::from_slice(&body_bytes)?;
        if json.get("result").is_some() {
            endpoint.set_healthy(true);
            Ok(())
        } else {
            endpoint.set_healthy(false);
            Err(Error::Config("No result in eth_blockNumber response".to_string()))
        }
    }

    // WebSocket health check: Send ping and expect pong
    async fn check_websocket(&self, endpoint: &mut Endpoint) -> Result<()> {
        // Connect to WebSocket
        let (ws_stream, _) = tokio::time::timeout(self.timeout, connect_async(&endpoint.url)).await??;
        let (mut write, mut read) = ws_stream.split();

        // Send ping
        write.send(Message::Ping(vec![])).await?;

        // Wait for pong
        let response = tokio::time::timeout(self.timeout, read.next()).await?;
        match response {
            Some(Ok(Message::Pong(_))) => {
                endpoint.set_healthy(true); 
                Ok(())
            }
            _ => {
                endpoint.set_healthy(false);
                Err(Error::Config("No pong received".to_string()))
            }
        }
    }

}