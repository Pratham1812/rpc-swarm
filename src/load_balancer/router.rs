use crate::error::Result;
use crate::load_balancer::{algorithms, endpoint};
use std::sync::atomic::AtomicUsize;

pub struct Router {
    path: String,
    network: String,
    active_connections: AtomicUsize,
    connections: Vec<Endpoints>,
    algorithm: LCA,
}

impl Router {
    pub fn new(path: String, network: String, connections: &Vec<Endpoints>) -> Result<Self> {
        Ok(Router {
            path,
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
