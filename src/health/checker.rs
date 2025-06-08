use crate::load_balancer::endpoint::Endpoint;
use crate::error::Result;

pub struct HealthChecker;

impl HealthChecker{

    pub async fn check_health(&self, endpoint: &mut Endpoint) -> Result<>{
        // ToDo implement health check logic
        // For HTTP -> make a JSON RPC request using eth_getBlockNumber
        // For websocket -> make a ping pong request
        Ok(())
    }
}
