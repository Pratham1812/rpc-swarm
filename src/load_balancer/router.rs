use crate::error::Result;
use crate::load_balancer::{LCA, Endpoint};
use std::sync::atomic::AtomicUsize;

pub struct Router {
    pub network: String,
    pub active_connections: AtomicUsize,
    pub connections: Vec<Endpoint>,
    pub algorithm: LCA,
}

impl Router {
    pub fn new(network: String, connections: Vec<Endpoint>) -> Result<Self> {
        Ok(Router {
            network,
            active_connections: AtomicUsize::new(0),
            connections,
            algorithm: LCA,
        })
    }

    pub fn route(&self) -> String {
        self.algorithm
            .select_endpoint(&self.connections, self.network.clone())
            .map_or_else(
                || "No healthy endpoint found".to_string(),
                |endpoint| endpoint.url.to_string(),
            )
    }
}
