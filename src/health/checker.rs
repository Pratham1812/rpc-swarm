// Updated imports for hyper v1.x ecosystem
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Body};
use hyper::Request;
use hyper_util::rt::TokioExecutor;

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;

use crate::load_balancer::endpoint::Endpoint;
use crate::error::{Error, Result};

pub struct HealthChecker {
    // The client type is now more specific, using components from hyper-util
    http_client: Client<HttpConnector, Full<Bytes>>,
    timeout: Duration,
}

impl HealthChecker {
    pub fn new(timeout_secs: u64) -> Self {
        // The new way to build a basic client
        let http_client = Client::builder(TokioExecutor::new()).build_http();
        let timeout = Duration::from_secs(timeout_secs);
        HealthChecker { http_client, timeout }
    }

    pub async fn check_health(&self, endpoint: &mut Endpoint) -> Result<()> {
        match endpoint.url.scheme() {
            "http" | "https" => self.check_http_health(endpoint).await,
            "ws" | "wss" => self.check_ws_health(endpoint).await,
            _ => Err(Error::HealthCheck(format!("Unsupported URL scheme: {}", endpoint.url))),
        }
    }

    // HTTP health check: Send eth_blockNumber JSON-RPC request
    async fn check_http_health(&self, endpoint: &mut Endpoint) -> Result<()> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });
        
        // The body must now be created using a concrete type like `Full` from http-body-util
        let req = Request::builder()
            .method("POST")
            .uri(endpoint.url.as_str())
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(request_body.to_string())))
            .map_err(|e| Error::HealthCheck(format!("Failed to build request: {}", e)))?;

        let response_future = self.http_client.request(req);
        let response = tokio::time::timeout(self.timeout, response_future)
            .await
            .map_err(|_| Error::HealthCheck("HTTP request timeout".to_string()))?
            .map_err(|e| Error::HealthCheck(format!("HTTP request failed: {}", e)))?;
        
        let status = response.status();
        
        if !status.is_success() {
            endpoint.set_healthy(false);
            return Err(Error::HealthCheck(format!("HTTP check failed with status: {}", status)));
        }

        // The new, simpler way to read a response body to bytes
        let body_bytes = response.into_body().collect().await
            .map_err(|e| Error::HealthCheck(format!("Failed to read response body: {}", e)))?
            .to_bytes();
        
        let json: Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| Error::HealthCheck(format!("Failed to parse JSON response: {}", e)))?;
        
        if json.get("result").is_some() {
            endpoint.set_healthy(true);
            Ok(())
        } else {
            endpoint.set_healthy(false);
            Err(Error::HealthCheck("No result in eth_blockNumber response".to_string()))
        }
    }

    // This function already used modern APIs and did not require changes.
    async fn check_ws_health(&self, endpoint: &mut Endpoint) -> Result<()> {
        let (ws_stream, _) = tokio::time::timeout(
            self.timeout, 
            connect_async(endpoint.url.as_str())
        )
        .await
        .map_err(|_| Error::HealthCheck("WebSocket connection timeout".to_string()))?
        .map_err(|e| Error::HealthCheck(format!("WebSocket connection failed: {}", e)))?;
        
        let (mut write, mut read) = ws_stream.split();

        if let Err(e) = write.send(Message::Ping(vec![])).await {
            endpoint.set_healthy(false);
            return Err(Error::HealthCheck(format!("Failed to send ping: {}", e)));
        }

        let response = tokio::time::timeout(self.timeout, read.next())
            .await
            .map_err(|_| Error::HealthCheck("WebSocket ping timeout".to_string()))?;
        
        match response {
            Some(Ok(Message::Pong(_))) => {
                endpoint.set_healthy(true);
                Ok(())
            }
            Some(Ok(msg)) => {
                endpoint.set_healthy(false);
                Err(Error::HealthCheck(format!("Expected pong, got: {:?}", msg)))
            }
            Some(Err(e)) => {
                endpoint.set_healthy(false);
                Err(Error::HealthCheck(format!("WebSocket error: {}", e)))
            }
            None => {
                endpoint.set_healthy(false);
                Err(Error::HealthCheck("WebSocket connection closed".to_string()))
            }
        }
    }
}

// The helper function `read_body_to_bytes` is no longer needed and can be deleted.
// The `BodyExt::collect()` method serves as its modern replacement.