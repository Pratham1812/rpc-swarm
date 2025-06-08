use crate::load_balancer::endpoint::Endpoint;

pub struct LCA;

//Least Connection Algorithm (LCA)
impl LCA {
    pub fn select_endpoint<'a>(
        &self,
        endpoints: &'a [Endpoint],
        network: String,
    ) -> Option<&'a Endpoint> {
        endpoints
            .iter()
            .filter(|e| e.is_healthy())
            .filter(|e| e.network == network)
            .min_by_key(|e| e.get_connections())
    }
}
