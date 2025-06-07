use log::{info, debug, error, warn};
use rpc_swarm::error::Result;

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init();
    info!("Swarm is starting...");
    // Your application logic goes here

    Ok(())
}