use crate::error::ProxyError;
use crate::load_balancer;
use crate::state::AppState;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::Arc;
use std::time::Instant;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody>, ProxyError> {
    let start_time = Instant::now();

    // Parse network from path (e.g., /ethereum/*, /base/*)
    let path = req.uri().path();
    let network_name = parse_network_from_path(path)?;

    // Get network state
    let network_state = state
        .get_network(&network_name)
        .ok_or_else(|| ProxyError::NetworkNotFound(network_name.clone()))?;

    // Increment total requests
    network_state.metrics.increment_total();

    // Select healthy endpoint
    let endpoint = match load_balancer::select_endpoint(&network_name, &network_state).await {
        Ok(ep) => ep,
        Err(e) => {
            network_state.metrics.increment_failure();
            tracing::error!(
                network = %network_name,
                error = %e,
                duration_ms = start_time.elapsed().as_millis(),
                "Failed to select endpoint"
            );
            return Err(e);
        }
    };

    tracing::info!(
        network = %network_name,
        endpoint = %endpoint,
        method = %req.method(),
        path = %path,
        "Forwarding request"
    );

    // Forward request
    match forward_request(req, &endpoint, &network_name).await {
        Ok(response) => {
            network_state.metrics.increment_success();
            let duration = start_time.elapsed();
            tracing::info!(
                network = %network_name,
                endpoint = %endpoint,
                status = %response.status(),
                duration_ms = duration.as_millis(),
                "Request completed"
            );
            Ok(response)
        }
        Err(e) => {
            network_state.metrics.increment_failure();
            tracing::error!(
                network = %network_name,
                endpoint = %endpoint,
                error = %e,
                duration_ms = start_time.elapsed().as_millis(),
                "Request failed"
            );
            Err(e)
        }
    }
}

fn parse_network_from_path(path: &str) -> Result<String, ProxyError> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Err(ProxyError::InvalidPath(
            "Path must start with network name (e.g., /ethereum)".to_string(),
        ));
    }

    Ok(parts[0].to_string())
}

async fn forward_request(
    mut req: Request<hyper::body::Incoming>,
    endpoint: &str,
    network_name: &str,
) -> Result<Response<BoxBody>, ProxyError> {
    // Build target URI (strip network prefix from path)
    let original_path = req.uri().path();
    let path_without_network = original_path
        .trim_start_matches('/')
        .splitn(2, '/')
        .nth(1)
        .unwrap_or("");

    let target_path = if path_without_network.is_empty() {
        "/"
    } else {
        path_without_network
    };

    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let target_uri = format!("{}{}{}", endpoint, target_path, query).parse()?;

    *req.uri_mut() = target_uri;

    // Filter hop-by-hop headers
    let headers = req.headers_mut();
    headers.remove("connection");
    headers.remove("keep-alive");
    headers.remove("proxy-authenticate");
    headers.remove("proxy-authorization");
    headers.remove("te");
    headers.remove("trailers");
    headers.remove("transfer-encoding");
    headers.remove("upgrade");

    // Create HTTPS-capable HTTP client
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .expect("Failed to load native roots")
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let client = Client::builder(TokioExecutor::new()).build(https);

    // Forward request
    let response = client.request(req).await?;

    // Convert response body to BoxBody
    let (parts, body) = response.into_parts();
    let boxed_body = body.map_err(|e| e).boxed();

    Ok(Response::from_parts(parts, boxed_body))
}

pub fn create_error_response(status: StatusCode, message: &str) -> Response<BoxBody> {
    let body = Full::new(Bytes::from(message.to_string()))
        .map_err(|e| match e {})
        .boxed();

    Response::builder().status(status).body(body).unwrap()
}
