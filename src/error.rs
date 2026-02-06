use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("All endpoints unhealthy for network '{0}'")]
    NoHealthyEndpoints(String),

    #[error("Network '{0}' not found in configuration")]
    NetworkNotFound(String),

    #[error("Invalid request path: {0}")]
    InvalidPath(String),

    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] hyper::Error),

    #[error("HTTP client error: {0}")]
    HttpClientError(#[from] hyper_util::client::legacy::Error),

    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
}
