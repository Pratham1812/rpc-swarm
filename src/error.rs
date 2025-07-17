use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Health Check error: {0}")]
    HealthCheck(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;