use crate::bpv7::EndpointId;
use crate::store::bundle_descriptor::BundleDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ClaPeer: Send + Sync {
    fn get_peer_endpoint_id(&self) -> EndpointId;

    /// Check if this peer is currently reachable/connectable
    /// This method should be implemented by each CLA type to perform
    /// appropriate connectivity checks (TCP ping, BLE scan, etc.)
    async fn is_reachable(&self) -> bool;

    /// Get the CLA type for this peer (e.g., "tcp", "ble", "udp")
    fn get_cla_type(&self) -> &str;

    /// Get the connection address/identifier for this peer
    fn get_connection_address(&self) -> String;
}

/// Represents a route entry in the routing table
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub destination: EndpointId,
    pub next_hop: EndpointId,
    pub cla_type: String,
    pub cost: u32,
    pub is_active: bool,
}

/// Routing table that maps destinations to next hops and CLAs
#[derive(Debug, Default)]
pub struct RoutingTable {
    routes: HashMap<EndpointId, Vec<RouteEntry>>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, entry: RouteEntry) {
        self.routes
            .entry(entry.destination.clone())
            .or_default()
            .push(entry);
    }

    pub fn get_routes_for_destination(&self, destination: &EndpointId) -> Vec<&RouteEntry> {
        self.routes
            .get(destination)
            .map(|routes| routes.iter().filter(|r| r.is_active).collect())
            .unwrap_or_default()
    }

    pub fn get_all_routes(&self) -> Vec<&RouteEntry> {
        self.routes
            .values()
            .flatten()
            .filter(|r| r.is_active)
            .collect()
    }

    /// Find the best route for a destination
    pub fn find_best_route(&self, destination: &EndpointId) -> Option<&RouteEntry> {
        self.get_routes_for_destination(destination)
            .into_iter()
            .min_by_key(|route| route.cost)
    }
}

#[async_trait]
pub trait RoutingAlgorithm: Send + Sync {
    fn notify_new_bundle(&mut self, descriptor: &BundleDescriptor);
    fn select_peers_for_forwarding<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_peers: &'a [Box<dyn ClaPeer>],
    ) -> Vec<&'a dyn ClaPeer>;

    /// Async version that checks connectivity before selecting peers
    async fn select_peers_for_forwarding_async<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_peers: &'a [Box<dyn ClaPeer>],
    ) -> Vec<&'a dyn ClaPeer> {
        // Default implementation falls back to sync version
        self.select_peers_for_forwarding(descriptor, all_peers)
    }

    /// New method: select routes based on routing table
    fn select_routes_for_forwarding(
        &self,
        descriptor: &BundleDescriptor,
        routing_table: &RoutingTable,
    ) -> Vec<RouteEntry>;
}

#[derive(Debug)]
pub enum RoutingAlgorithmType {
    Epidemic,
    Prophet,
    // SprayAndWait,
}

pub struct RoutingConfig {
    pub algorithm_type: RoutingAlgorithmType,
}

impl RoutingConfig {
    pub fn new(algorithm_type: RoutingAlgorithmType) -> Self {
        Self { algorithm_type }
    }

    pub fn create_algorithm(&self) -> Box<dyn RoutingAlgorithm> {
        match self.algorithm_type {
            RoutingAlgorithmType::Epidemic => Box::new(crate::routing::epidemic::EpidemicRouting),
            RoutingAlgorithmType::Prophet => {
                // TODO: Implement Prophet routing algorithm
                // For now, fall back to epidemic
                eprintln!("Warning: Prophet routing not yet implemented, falling back to epidemic");
                Box::new(crate::routing::epidemic::EpidemicRouting)
            }
        }
    }
}
