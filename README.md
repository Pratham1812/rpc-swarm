# RPC Swarm - Multichain RPC Load Balancer

A high-performance, production-ready RPC load balancer built in Rust that supports multiple blockchain networks with automatic health checking and failover capabilities.

## Features

- **Multi-Network Support**: Ethereum, Base, Arbitrum (easily extensible)
- **Round-Robin Load Balancing**: Lock-free atomic counter-based distribution
- **Health Monitoring**: Asynchronous health checks using `eth_blockNumber` JSON-RPC
- **Automatic Failover**: Intelligent endpoint selection skipping unhealthy RPCs
- **Path-Based Routing**: Route requests by network via URL path (e.g., `/ethereum`, `/base`)
- **Request Metrics**: Per-network atomic counters for total/success/failure tracking
- **Graceful Shutdown**: Proper signal handling for clean termination
- **Comprehensive Logging**: Detailed tracing of endpoint selection, health transitions, and request timing

## Architecture

```
Client Request
     ↓
HTTP Proxy Server (Hyper)
     ↓
Path Parser (/ethereum, /base, /arbitrum)
     ↓
Round-Robin Load Balancer
     ↓
Health Check Filter (skip unhealthy endpoints)
     ↓
Request Forwarding to Selected RPC
     ↓
Response Streaming Back to Client
```

### Key Components

- **Config Module**: TOML-based configuration with environment override support
- **State Module**: Shared application state with `Arc<RwLock<>>` for health status
- **Health Check Module**: Background tasks checking endpoints every N seconds
- **Load Balancer**: Atomic counter-based round-robin selection
- **Proxy Module**: HTTP request forwarding with header filtering
- **Metrics Module**: Lock-free atomic counters for observability

## Configuration

### config.toml

```toml
[server]
port = 8080

[health_check]
interval_secs = 30

[networks.ethereum]
endpoints = [
    "https://eth.llamarpc.com",
    "https://rpc.ankr.com/eth"
]

[networks.base]
endpoints = [
    "https://mainnet.base.org",
    "https://base.llamarpc.com"
]
```

### Environment Variables

Create a `.env` file for overrides (optional):

```bash
# Override config.toml values with RPC_ prefix
RPC_SERVER__PORT=9000
RPC_HEALTH_CHECK__INTERVAL_SECS=60

# Set logging level
RUST_LOG=debug
```

## Usage

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

Or run the binary directly:

```bash
./target/release/rpc-swarm
```

### Making Requests

Send JSON-RPC requests to the proxy with the network in the path:

```bash
# Ethereum request
curl -X POST http://localhost:8080/ethereum \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'

# Base request
curl -X POST http://localhost:8080/base \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_blockNumber",
    "params": [],
    "id": 1
  }'
```

## Health Checking

- Each endpoint is checked every `interval_secs` (default: 30s)
- Health check uses `eth_blockNumber` JSON-RPC call
- Endpoints are marked unhealthy after **3 consecutive failures**
- Endpoints recover immediately on successful health check
- Health transitions are logged for visibility

## Logging

Set the `RUST_LOG` environment variable to control log levels:

```bash
RUST_LOG=info cargo run      # Info and above
RUST_LOG=debug cargo run     # Debug and above
RUST_LOG=trace cargo run     # All logs
```

Logged events:

- Endpoint selection for each request
- Request duration in milliseconds
- Health status transitions (healthy ↔ unhealthy)
- HTTP errors and failures

## Project Structure

```
rpc-swarm/
├── Cargo.toml              # Dependencies and project metadata
├── config.toml             # Network and server configuration
├── .env.example            # Example environment variables
├── src/
│   ├── main.rs            # Server initialization and shutdown
│   ├── config.rs          # Configuration loading and validation
│   ├── state.rs           # Shared application state
│   ├── error.rs           # Custom error types
│   ├── metrics.rs         # Atomic counter metrics
│   ├── health_check.rs    # Background health monitoring
│   ├── load_balancer.rs   # Round-robin endpoint selection
│   └── proxy.rs           # HTTP request forwarding
└── README.md
```

## Development

### Dependencies

- **hyper 1.0**: HTTP server and client
- **tokio**: Async runtime
- **reqwest**: Health check HTTP client
- **serde**: Configuration serialization
- **config**: TOML configuration parsing
- **tracing**: Structured logging

### Adding New Networks

1. Add network configuration to `config.toml`:

```toml
[networks.polygon]
endpoints = [
    "https://polygon-rpc.com",
    "https://polygon.llamarpc.com"
]
```

2. Restart the server - the network is automatically detected and initialized

### Commit Structure

This project follows conventional commits for learning purposes:

- `feat(module): description` - New features
- `fix(module): description` - Bug fixes
- `refactor(module): description` - Code improvements
- `docs: description` - Documentation updates

## Performance Considerations

- **Lock-Free Operations**: Round-robin counter uses `AtomicUsize` with `Ordering::Relaxed`
- **Read-Heavy Optimization**: Health status uses `RwLock` (multiple readers, single writer)
- **Async I/O**: Non-blocking request handling with Tokio
- **Connection Pooling**: Hyper client reuses connections
- **Zero-Copy**: Body streaming without buffering

## Error Handling

- `NoHealthyEndpoints`: Returns 503 Service Unavailable
- `NetworkNotFound`: Returns 404 Not Found
- `InvalidPath`: Returns 400 Bad Request
- HTTP forwarding errors: Returns 502 Bad Gateway

## License

MIT

## Contributing

This is a learning project demonstrating Rust async patterns, HTTP proxying, and production-ready architecture. Feel free to fork and experiment!
