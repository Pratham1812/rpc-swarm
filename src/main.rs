mod config;
mod error;
mod health_check;
mod load_balancer;
mod metrics;
mod proxy;
mod state;

use anyhow::{Context, Result};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting RPC Swarm load balancer");

    // Load and validate configuration
    let config = config::Config::load().context("Failed to load configuration")?;
    config.validate().context("Invalid configuration")?;

    tracing::info!(
        port = config.server.port,
        networks = config.networks.len(),
        health_check_interval = config.health_check.interval_secs,
        "Configuration loaded"
    );

    // Initialize application state
    let state = Arc::new(AppState::new(&config));

    // Spawn health checkers for each network
    for (network_name, network_state) in &state.networks {
        tracing::info!(
            network = %network_name,
            endpoints = network_state.endpoints.len(),
            "Initializing network"
        );

        health_check::spawn_health_checker(
            network_name.clone(),
            Arc::clone(network_state),
            config.health_check.interval_secs,
        )
        .await;
    }

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind server")?;

    tracing::info!("Server listening on {}", addr);

    // Graceful shutdown setup
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("Shutdown signal received");
    };

    // Server loop
    tokio::select! {
        result = serve(listener, state) => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            tracing::info!("Shutting down gracefully");
        }
    }

    Ok(())
}

async fn serve(listener: TcpListener, state: Arc<AppState>) -> Result<()> {
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let state = Arc::clone(&state);
                async move {
                    match proxy::handle_request(req, state).await {
                        Ok(response) => Ok::<_, hyper::Error>(response),
                        Err(e) => {
                            tracing::error!("Request handling error: {}", e);
                            let status = match e {
                                error::ProxyError::NetworkNotFound(_) => StatusCode::NOT_FOUND,
                                error::ProxyError::NoHealthyEndpoints(_) => {
                                    StatusCode::SERVICE_UNAVAILABLE
                                }
                                error::ProxyError::InvalidPath(_) => StatusCode::BAD_REQUEST,
                                _ => StatusCode::BAD_GATEWAY,
                            };
                            Ok(proxy::create_error_response(status, &e.to_string()))
                        }
                    }
                }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                tracing::error!("Connection error from {}: {}", remote_addr, err);
            }
        });
    }
}
