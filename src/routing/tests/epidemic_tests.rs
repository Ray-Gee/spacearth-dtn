use crate::bpv7::bundle::Bundle;
use crate::bpv7::EndpointId;
use crate::routing::algorithm::{
    ConvergenceSender, RouteEntry, RoutingAlgorithm, RoutingTable, TcpSender,
};
use crate::routing::epidemic::EpidemicRouting;
use crate::store::bundle_descriptor::BundleDescriptor;

#[test]
fn test_epidemic_routing_default() {
    let routing = EpidemicRouting;
    assert!(matches!(routing, EpidemicRouting));
}

#[test]
fn test_epidemic_routing_direct() {
    let routing = EpidemicRouting;
    assert!(matches!(routing, EpidemicRouting));
}

#[test]
fn test_notify_new_bundle() {
    let mut routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    // Should not panic or error
    routing.notify_new_bundle(&descriptor);
}

#[test]
fn test_select_peers_for_forwarding() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    let sender1: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer1")));
    let sender2: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer2")));

    let all_senders = vec![sender1, sender2];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_senders);

    assert_eq!(selected.len(), 2);
}

#[test]
fn test_select_peers_for_forwarding_with_already_sent() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let mut descriptor = BundleDescriptor::new(bundle);

    // Mark one peer as already sent
    descriptor.mark_sent(crate::bpv7::EndpointId::from("dtn://peer1"));

    let sender1: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer1")));
    let sender2: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer2")));

    let all_senders = vec![sender1, sender2];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_senders);

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].get_peer_endpoint_id().as_str(), "dtn://peer2");
}

#[test]
fn test_select_peers_for_forwarding_empty_senders() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    let all_senders: Vec<Box<dyn ConvergenceSender>> = vec![];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_senders);

    assert_eq!(selected.len(), 0);
}

#[test]
fn test_select_peers_for_forwarding_duplicate_endpoints() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    // Create multiple senders with the same endpoint ID
    let sender1: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer1")));
    let sender2: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer1")));
    let sender3: Box<dyn ConvergenceSender> =
        Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer2")));

    let all_senders = vec![sender1, sender2, sender3];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_senders);

    // Should only select unique endpoints
    assert_eq!(selected.len(), 2);
}

#[test]
fn test_select_routes_for_forwarding_empty_table() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let routing_table = RoutingTable::new();

    let selected = routing.select_routes_for_forwarding(&descriptor, &routing_table);
    assert_eq!(selected.len(), 0);
}

#[test]
fn test_select_routes_for_forwarding_with_routes() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let mut routing_table = RoutingTable::new();

    // Add some routes
    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    });

    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://other"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    });

    let selected = routing.select_routes_for_forwarding(&descriptor, &routing_table);
    assert_eq!(selected.len(), 2); // Epidemic routing forwards to all available routes
}

#[test]
fn test_select_routes_for_forwarding_with_already_sent() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let mut descriptor = BundleDescriptor::new(bundle);
    let mut routing_table = RoutingTable::new();

    // Mark one next hop as already sent
    descriptor.mark_sent(EndpointId::from("dtn://router1"));

    // Add routes
    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    });

    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://other"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    });

    let selected = routing.select_routes_for_forwarding(&descriptor, &routing_table);
    assert_eq!(selected.len(), 1); // Should exclude already sent route
    assert_eq!(selected[0].next_hop.as_str(), "dtn://router2");
}

#[test]
fn test_select_routes_for_forwarding_inactive_routes() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let mut routing_table = RoutingTable::new();

    // Add an inactive route
    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: false,
    });

    let selected = routing.select_routes_for_forwarding(&descriptor, &routing_table);
    assert_eq!(selected.len(), 0); // Inactive routes should not be selected
}

#[test]
fn test_select_routes_for_forwarding_duplicate_next_hops() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let mut routing_table = RoutingTable::new();

    // Add routes with the same next hop
    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest1"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    });

    routing_table.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest2"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    });

    let selected = routing.select_routes_for_forwarding(&descriptor, &routing_table);
    assert_eq!(selected.len(), 1); // Should only select unique next hops
    assert_eq!(selected[0].next_hop.as_str(), "dtn://router1");
}
