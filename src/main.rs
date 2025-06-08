use crate::error::Result;
use log::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Swarm is starting...");
    // Your application logic goes here

    Ok(())
}
