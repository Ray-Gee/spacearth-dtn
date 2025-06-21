use crate::routing::algorithm::{ConvergenceSender, RouteEntry, RoutingAlgorithm, RoutingTable};
use crate::store::bundle_descriptor::BundleDescriptor;
use std::collections::HashSet;

#[derive(Default)]
pub struct EpidemicRouting;

impl RoutingAlgorithm for EpidemicRouting {
    fn notify_new_bundle(&mut self, _descriptor: &BundleDescriptor) {
        // Do nothing for epidemic routing
        // In a more sophisticated implementation, this could track bundle metadata
    }

    fn select_peers_for_forwarding<'a>(
        &self,
        descriptor: &BundleDescriptor,
        all_senders: &'a [Box<dyn ConvergenceSender>],
    ) -> Vec<&'a dyn ConvergenceSender> {
        let mut seen_eids = HashSet::new();
        let mut result = Vec::new();

        for sender in all_senders {
            let eid = sender.get_peer_endpoint_id();
            if !descriptor.has_been_sent_to(&eid) && seen_eids.insert(eid.clone()) {
                result.push(&**sender);
            }
        }

        result
    }

    fn select_routes_for_forwarding(
        &self,
        descriptor: &BundleDescriptor,
        routing_table: &RoutingTable,
    ) -> Vec<RouteEntry> {
        let mut result = Vec::new();
        let mut seen_next_hops = HashSet::new();

        // Get all available routes
        let all_routes = routing_table.get_all_routes();

        for route in all_routes {
            // For epidemic routing, forward to all available next hops
            // unless we've already sent to this next hop
            if !descriptor.has_been_sent_to(&route.next_hop)
                && seen_next_hops.insert(route.next_hop.clone())
            {
                result.push(route.clone());
            }
        }

        result
    }
}
