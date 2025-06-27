use crate::bpv7::bundle::Bundle;
use crate::bpv7::EndpointId;
use crate::cla::peer::ClaPeer;
use crate::cla::TcpPeer;
use crate::routing::algorithm::{
    RouteEntry, RoutingAlgorithm, RoutingAlgorithmType, RoutingConfig, RoutingTable,
};
use crate::routing::epidemic::EpidemicRouting;
use crate::store::bundle_descriptor::BundleDescriptor;

#[test]
fn test_tcp_peer_new() {
    let eid = EndpointId::from("dtn://test");
    let peer = TcpPeer::new(eid.clone(), "127.0.0.1:8080".to_string());
    assert_eq!(peer.peer_id, eid);
}

#[test]
fn test_tcp_peer_get_peer_endpoint_id() {
    let eid = EndpointId::from("dtn://test");
    let peer = TcpPeer::new(eid.clone(), "127.0.0.1:8080".to_string());
    assert_eq!(peer.get_peer_endpoint_id(), eid);
}

#[test]
fn test_route_entry_creation() {
    let entry = RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    assert_eq!(entry.destination.as_str(), "dtn://dest");
    assert_eq!(entry.next_hop.as_str(), "dtn://router");
    assert_eq!(entry.cla_type, "tcp");
    assert_eq!(entry.cost, 10);
    assert!(entry.is_active);
}

#[test]
fn test_route_entry_clone() {
    let entry = RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let cloned = entry.clone();
    assert_eq!(entry.destination, cloned.destination);
    assert_eq!(entry.next_hop, cloned.next_hop);
    assert_eq!(entry.cla_type, cloned.cla_type);
    assert_eq!(entry.cost, cloned.cost);
    assert_eq!(entry.is_active, cloned.is_active);
}

#[test]
fn test_routing_table_new() {
    let table = RoutingTable::new();
    assert!(table.get_all_routes().is_empty());
}

#[test]
fn test_routing_table_default() {
    let table = RoutingTable::default();
    assert!(table.get_all_routes().is_empty());
}

#[test]
fn test_routing_table_add_route() {
    let mut table = RoutingTable::new();
    let entry = RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    table.add_route(entry.clone());
    let routes = table.get_routes_for_destination(&EndpointId::from("dtn://dest"));
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].destination, entry.destination);
}

#[test]
fn test_routing_table_multiple_routes_same_destination() {
    let mut table = RoutingTable::new();
    let dest = EndpointId::from("dtn://dest");

    let entry1 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let entry2 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    };

    table.add_route(entry1);
    table.add_route(entry2);

    let routes = table.get_routes_for_destination(&dest);
    assert_eq!(routes.len(), 2);
}

#[test]
fn test_routing_table_inactive_routes() {
    let mut table = RoutingTable::new();
    let dest = EndpointId::from("dtn://dest");

    let entry = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: false,
    };

    table.add_route(entry);

    let routes = table.get_routes_for_destination(&dest);
    assert_eq!(routes.len(), 0); // Inactive routes should be filtered out
}

#[test]
fn test_routing_table_find_best_route() {
    let mut table = RoutingTable::new();
    let dest = EndpointId::from("dtn://dest");

    let entry1 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let entry2 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    };

    table.add_route(entry1);
    table.add_route(entry2);

    let best = table.find_best_route(&dest);
    assert!(best.is_some());
    assert_eq!(best.unwrap().cost, 5); // Should return the route with lowest cost
}

#[test]
fn test_routing_table_find_best_route_no_routes() {
    let table = RoutingTable::new();
    let dest = EndpointId::from("dtn://nonexistent");

    let best = table.find_best_route(&dest);
    assert!(best.is_none());
}

#[test]
fn test_routing_table_get_all_routes() {
    let mut table = RoutingTable::new();

    let entry1 = RouteEntry {
        destination: EndpointId::from("dtn://dest1"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let entry2 = RouteEntry {
        destination: EndpointId::from("dtn://dest2"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    };

    table.add_route(entry1);
    table.add_route(entry2);

    let all_routes = table.get_all_routes();
    assert_eq!(all_routes.len(), 2);
}

#[test]
fn test_routing_algorithm_type_debug() {
    let epidemic = RoutingAlgorithmType::Epidemic;
    let prophet = RoutingAlgorithmType::Prophet;

    assert_eq!(format!("{epidemic:?}"), "Epidemic");
    assert_eq!(format!("{prophet:?}"), "Prophet");
}

#[test]
fn test_routing_config_new() {
    let config = RoutingConfig::new(RoutingAlgorithmType::Epidemic);
    assert!(matches!(
        config.algorithm_type,
        RoutingAlgorithmType::Epidemic
    ));
}

#[test]
fn test_routing_config_create_algorithm_epidemic() {
    let config = RoutingConfig::new(RoutingAlgorithmType::Epidemic);
    let algorithm = config.create_algorithm();

    // Test that we can create the algorithm
    let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let peers: Vec<Box<dyn ClaPeer>> = vec![];

    let selected = algorithm.select_peers_for_forwarding(&descriptor, &peers);
    assert!(selected.is_empty()); // No peers provided
}

#[test]
fn test_routing_config_create_algorithm_prophet() {
    let config = RoutingConfig::new(RoutingAlgorithmType::Prophet);
    let algorithm = config.create_algorithm();

    // Test that Prophet falls back to Epidemic
    let bundle = Bundle::new("dtn://src", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);
    let peers: Vec<Box<dyn ClaPeer>> = vec![];

    let selected = algorithm.select_peers_for_forwarding(&descriptor, &peers);
    assert!(selected.is_empty()); // No peers provided
}

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
fn routing_test_select_peers_for_forwarding() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    let peer1: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer1"),
    ));
    let peer2: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer2"),
    ));

    let all_peers = vec![peer1, peer2];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_peers);

    assert_eq!(selected.len(), 2);
}

#[test]
fn test_select_peers_for_forwarding_with_already_sent() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let mut descriptor = BundleDescriptor::new(bundle);

    // Mark one peer as already sent
    descriptor.mark_sent(crate::bpv7::EndpointId::from("dtn://peer1"));

    let peer1: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer1"),
    ));
    let peer2: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer2"),
    ));

    let all_peers = vec![peer1, peer2];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_peers);

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].get_peer_endpoint_id().as_str(), "dtn://peer2");
}

#[test]
fn test_select_peers_for_forwarding_empty_senders() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    let all_peers: Vec<Box<dyn ClaPeer>> = vec![];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_peers);

    assert_eq!(selected.len(), 0);
}

#[test]
fn test_select_peers_for_forwarding_duplicate_endpoints() {
    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    // Create multiple senders with the same endpoint ID
    let peer1: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer1"),
    ));
    let peer2: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer1"),
    ));
    let peer3: Box<dyn ClaPeer> = Box::new(TcpPeer::from_endpoint_id(
        crate::bpv7::EndpointId::from("dtn://peer2"),
    ));

    let all_peers = vec![peer1, peer2, peer3];
    let selected = routing.select_peers_for_forwarding(&descriptor, &all_peers);

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
    assert_eq!(selected.len(), 0); // Epidemic routing does not use routing table
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
    assert_eq!(selected.len(), 0); // Epidemic routing does not use routing table
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
    assert_eq!(selected.len(), 0); // Epidemic routing does not use routing table
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
    assert_eq!(selected.len(), 0); // Epidemic routing does not use routing table
}

#[tokio::test]
async fn routing_test_select_peers_for_forwarding_async() {
    use crate::bpv7::EndpointId;
    use crate::cla::ClaPeer;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct DummyPeer {
        eid: EndpointId,
        reachable: bool,
    }

    #[async_trait]
    impl ClaPeer for DummyPeer {
        fn get_peer_endpoint_id(&self) -> EndpointId {
            self.eid.clone()
        }
        fn get_connection_address(&self) -> String {
            "dummy".to_string()
        }
        fn get_cla_type(&self) -> &str {
            "dummy"
        }
        async fn is_reachable(&self) -> bool {
            self.reachable
        }
        fn clone_box(&self) -> Box<dyn ClaPeer> {
            Box::new(self.clone())
        }
        async fn activate(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    let routing = EpidemicRouting;
    let bundle = Bundle::new("dtn://source", "dtn://dest", b"test".to_vec());
    let descriptor = BundleDescriptor::new(bundle);

    let peer1: Box<dyn ClaPeer> = Box::new(DummyPeer {
        eid: EndpointId::from("dtn://peer1"),
        reachable: true,
    });
    let peer2: Box<dyn ClaPeer> = Box::new(DummyPeer {
        eid: EndpointId::from("dtn://peer2"),
        reachable: false,
    });

    let all_peers = vec![peer1, peer2];
    let selected = routing
        .select_peers_for_forwarding_async(&descriptor, &all_peers)
        .await;

    // Only reachable peer should be selected
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].get_peer_endpoint_id().as_str(), "dtn://peer1");
}
