use crate::routing::algorithm::{ClaPeer, RouteEntry, RoutingAlgorithm, RoutingTable};
use crate::store::bundle_descriptor::BundleDescriptor;
use async_trait::async_trait;
use std::collections::HashSet;

/// Epidemic Routing Algorithm
///
/// Epidemic routing is a simple flooding-based approach where:
/// - Bundles are forwarded to ALL available peers (except those already sent to)
/// - No routing table is needed - routing decisions are based on peer availability
/// - The goal is maximum delivery probability at the cost of network overhead
/// - Each peer acts as a potential relay for all bundles
#[derive(Default)]
pub struct EpidemicRouting;

#[async_trait]
impl RoutingAlgorithm for EpidemicRouting {
    fn notify_new_bundle(&mut self, _descriptor: &BundleDescriptor) {
        // Epidemic routing doesn't need to track bundle metadata
        // All bundles are forwarded to all available peers regardless of destination
    }

    fn select_peers_for_forwarding<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_peers: &'a [Box<dyn ClaPeer>],
    ) -> Vec<&'a dyn ClaPeer> {
        // Epidemic routing: forward to ALL available peers (except those already sent to)
        // This is the core of epidemic routing - no routing decisions, just flood to everyone
        let mut seen_eids = HashSet::new();
        let mut result = Vec::new();

        for peer in all_peers {
            let eid = peer.get_peer_endpoint_id();
            if !descriptor.has_been_sent_to(&eid) && seen_eids.insert(eid.clone()) {
                result.push(&**peer);
            }
        }

        result
    }

    /// Async version that checks connectivity before selecting peers
    async fn select_peers_for_forwarding_async<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_peers: &'a [Box<dyn ClaPeer>],
    ) -> Vec<&'a dyn ClaPeer> {
        // Epidemic routing with connectivity check: forward to ALL reachable peers
        let mut seen_eids = HashSet::new();
        let mut result = Vec::new();

        for peer in all_peers {
            let eid = peer.get_peer_endpoint_id();
            if !descriptor.has_been_sent_to(&eid)
                && seen_eids.insert(eid.clone())
                && peer.is_reachable().await
            {
                result.push(&**peer);
            }
        }

        result
    }

    fn select_routes_for_forwarding(
        &self,
        _descriptor: &BundleDescriptor,
        _routing_table: &RoutingTable,
    ) -> Vec<RouteEntry> {
        // Epidemic routing doesn't use routing tables
        // It forwards directly to available peers via select_peers_for_forwarding
        Vec::new()
    }
}
