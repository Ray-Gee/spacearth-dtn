use crate::api::{node::DtnNode, BundleStatus};
use crate::bpv7::EndpointId;
use crate::routing::algorithm::{RouteEntry, RoutingAlgorithmType, RoutingConfig};
use tempfile::TempDir;

#[test]
fn test_dtn_node_new() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let _node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    // Test that node is created successfully
    // Note: store_path is private, so we can't directly test it
    Ok(())
}

#[test]
fn test_dtn_node_default() {
    let _node = DtnNode::default();
    // Note: store_path is private, so we can't directly test it
}

#[test]
fn test_dtn_node_with_config() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let _node = DtnNode::with_config(Some(temp_dir.path().to_str().unwrap()))?;

    // Note: store_path is private, so we can't directly test it
    Ok(())
}

#[test]
fn test_dtn_node_with_config_default_path() -> anyhow::Result<()> {
    let _node = DtnNode::with_config(None)?;
    // Note: store_path is private, so we can't directly test it
    Ok(())
}

#[test]
fn test_dtn_node_with_routing_algorithm() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let routing_config = RoutingConfig::new(RoutingAlgorithmType::Epidemic);
    let _node = DtnNode::with_routing_algorithm(temp_dir.path().to_str().unwrap(), routing_config)?;

    // Note: store_path is private, so we can't directly test it
    Ok(())
}

#[test]
fn test_insert_bundle() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Test message".to_string())?;

    let bundles = node.list_bundles()?;
    assert_eq!(bundles.len(), 1);
    Ok(())
}

#[test]
fn test_insert_multiple_bundles() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Message 1".to_string())?;
    node.insert_bundle("Message 2".to_string())?;
    node.insert_bundle("Message 3".to_string())?;

    let bundles = node.list_bundles()?;
    assert_eq!(bundles.len(), 3);
    Ok(())
}

#[test]
fn test_show_bundle() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let message = "Test message for show";
    node.insert_bundle(message.to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();

    let bundle = node.show_bundle(bundle_id)?;
    assert_eq!(bundle.payload, message.as_bytes());
    Ok(())
}

#[test]
fn test_add_route() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let route = RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    node.add_route(route.clone())?;

    let routes = node.get_all_routes()?;
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].destination, route.destination);
    Ok(())
}

#[test]
fn test_add_multiple_routes() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let route1 = RouteEntry {
        destination: EndpointId::from("dtn://dest1"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let route2 = RouteEntry {
        destination: EndpointId::from("dtn://dest2"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    };

    node.add_route(route1)?;
    node.add_route(route2)?;

    let routes = node.get_all_routes()?;
    assert_eq!(routes.len(), 2);
    Ok(())
}

#[test]
fn test_find_best_route() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let dest = EndpointId::from("dtn://dest");

    let route1 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    };

    let route2 = RouteEntry {
        destination: dest.clone(),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    };

    node.add_route(route1)?;
    node.add_route(route2)?;

    let best_route = node.find_best_route(&dest)?;
    assert!(best_route.is_some());
    assert_eq!(best_route.unwrap().cost, 5); // Should be the cheaper route
    Ok(())
}

#[test]
fn test_find_best_route_no_routes() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    let dest = EndpointId::from("dtn://nonexistent");
    let best_route = node.find_best_route(&dest)?;
    assert!(best_route.is_none());
    Ok(())
}

#[test]
fn test_select_peers_for_forwarding() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Test message".to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();
    let bundle = node.show_bundle(bundle_id)?;

    let peers = node.select_peers_for_forwarding(&bundle)?;
    assert_eq!(peers.len(), 2); // Should return the dummy peers
    Ok(())
}

#[test]
fn test_select_routes_for_forwarding_empty_table() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Test message".to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();
    let bundle = node.show_bundle(bundle_id)?;

    let routes = node.select_routes_for_forwarding(&bundle)?;
    assert_eq!(routes.len(), 0); // No routes in table
    Ok(())
}

#[test]
fn test_select_routes_for_forwarding_with_routes() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    // Add some routes
    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    })?;

    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://other"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    })?;

    node.insert_bundle("Test message".to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();
    let bundle = node.show_bundle(bundle_id)?;

    let routes = node.select_routes_for_forwarding(&bundle)?;
    assert_eq!(routes.len(), 2); // Epidemic routing should select all routes
    Ok(())
}

#[test]
fn test_get_bundle_status_single() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Test message".to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();

    let status = node.get_bundle_status(Some(bundle_id))?;
    match status {
        BundleStatus::Single { id, bundle } => {
            assert_eq!(id, *bundle_id);
            assert_eq!(bundle.payload, b"Test message");
        }
        _ => panic!("Expected Single status"),
    }
    Ok(())
}

#[test]
fn test_get_bundle_status_summary() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Message 1".to_string())?;
    node.insert_bundle("Message 2".to_string())?;

    let status = node.get_bundle_status(None)?;
    match status {
        BundleStatus::Summary {
            active,
            expired,
            total,
        } => {
            assert_eq!(active, 2);
            assert_eq!(expired, 0);
            assert_eq!(total, 2);
        }
        _ => panic!("Expected Summary status"),
    }
    Ok(())
}

#[test]
fn test_cleanup_expired() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    node.insert_bundle("Test message".to_string())?;

    // Should not error even if no bundles are expired
    node.cleanup_expired()?;

    let bundles = node.list_bundles()?;
    assert_eq!(bundles.len(), 1); // Bundle should still be there
    Ok(())
}

#[test]
fn test_routing_with_prophet_algorithm() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let routing_config = RoutingConfig::new(RoutingAlgorithmType::Prophet);
    let node = DtnNode::with_routing_algorithm(temp_dir.path().to_str().unwrap(), routing_config)?;

    node.insert_bundle("Test message with Prophet".to_string())?;

    let bundles = node.list_bundles()?;
    assert_eq!(bundles.len(), 1);
    Ok(())
}

#[test]
fn test_complex_routing_scenario() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let node = DtnNode::with_store_path(temp_dir.path().to_str().unwrap())?;

    // Add multiple routes with different costs
    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://expensive-router"),
        cla_type: "tcp".to_string(),
        cost: 100,
        is_active: true,
    })?;

    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://cheap-router"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    })?;

    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://medium-router"),
        cla_type: "lora".to_string(),
        cost: 50,
        is_active: true,
    })?;

    // Insert bundle and test routing
    node.insert_bundle("Complex routing test".to_string())?;

    let bundles = node.list_bundles()?;
    let bundle_id = bundles.first().unwrap();
    let bundle = node.show_bundle(bundle_id)?;

    // Test route selection
    let routes = node.select_routes_for_forwarding(&bundle)?;
    assert_eq!(routes.len(), 3); // Epidemic should select all routes

    // Test best route finding
    let dest = EndpointId::from("dtn://dest");
    let best_route = node.find_best_route(&dest)?;
    assert!(best_route.is_some());
    assert_eq!(best_route.unwrap().cost, 5); // Should be the cheapest

    Ok(())
}
