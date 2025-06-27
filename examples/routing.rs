use sdtn::consts::BUNDLES_CUSTOM_ROUTING_DIR;
use sdtn::routing::algorithm::{RouteEntry, RoutingAlgorithmType, RoutingConfig};
use sdtn::{bpv7::EndpointId, DtnNode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧭 SpaceArth DTN Real Routing Example");

    // Method 1: Using default routing algorithm with routing table
    println!("\n📋 Method 1: Using routing table with epidemic routing");
    let node = DtnNode::new()?;

    // Add some routes to the routing table
    println!("  Adding routes to routing table...");
    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router1"),
        cla_type: "tcp".to_string(),
        cost: 10,
        is_active: true,
    })?;

    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://router2"),
        cla_type: "ble".to_string(),
        cost: 5,
        is_active: true,
    })?;

    node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://other-dest"),
        next_hop: EndpointId::from("dtn://router3"),
        cla_type: "lora".to_string(),
        cost: 15,
        is_active: true,
    })?;

    // Show all routes
    println!("  Current routing table:");
    let routes = node.get_all_routes()?;
    for (i, route) in routes.iter().enumerate() {
        println!(
            "    {}. {} -> {} via {} (cost: {}, cla: {})",
            i + 1,
            route.destination,
            route.next_hop,
            route.next_hop,
            route.cost,
            route.cla_type
        );
    }

    // Insert a test bundle
    node.insert_bundle("Test message for real routing".to_string())
        .await?;

    // Get the bundle and test routing
    let bundles = node.list_bundles()?;
    if let Some(id) = bundles.first() {
        let bundle = node.show_bundle(id)?;
        println!("  Testing routing for bundle: {id}");
        println!("  Destination: {}", bundle.primary.destination);

        // Test the new routing method using routing table
        match node.select_routes_for_forwarding(&bundle).await {
            Ok(routes) => {
                println!("  Selected {} routes for forwarding:", routes.len());
                for (i, route) in routes.iter().enumerate() {
                    println!(
                        "    {}. {} via {} (cost: {}, cla: {})",
                        i + 1,
                        route.next_hop,
                        route.next_hop,
                        route.cost,
                        route.cla_type
                    );
                }
            }
            Err(e) => {
                eprintln!("  ❌ Failed to select routes: {e}");
            }
        }

        // Test finding best route for destination
        let destination = EndpointId::from(&bundle.primary.destination);
        match node.find_best_route(&destination)? {
            Some(best_route) => {
                println!(
                    "  Best route to {}: {} via {} (cost: {}, cla: {})",
                    destination,
                    best_route.next_hop,
                    best_route.next_hop,
                    best_route.cost,
                    best_route.cla_type
                );
            }
            None => {
                println!("  No route found to {destination}");
            }
        }
    }

    // Method 2: Using custom routing algorithm with routing table
    println!("\n📋 Method 2: Using Prophet routing with routing table");
    let routing_config = RoutingConfig::new(RoutingAlgorithmType::Prophet);
    let custom_node = DtnNode::with_routing_algorithm(BUNDLES_CUSTOM_ROUTING_DIR, routing_config)?;

    // Add routes to custom node
    custom_node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://prophet-router1"),
        cla_type: "tcp".to_string(),
        cost: 8,
        is_active: true,
    })?;

    custom_node.add_route(RouteEntry {
        destination: EndpointId::from("dtn://dest"),
        next_hop: EndpointId::from("dtn://prophet-router2"),
        cla_type: "ble".to_string(),
        cost: 12,
        is_active: true,
    })?;

    // Insert a test bundle with custom routing
    custom_node
        .insert_bundle("Message with Prophet routing".to_string())
        .await?;

    // Test routing with custom algorithm
    let bundles = custom_node.list_bundles()?;
    if let Some(id) = bundles.first() {
        let bundle = custom_node.show_bundle(id)?;
        println!("  Testing Prophet routing for bundle: {id}");

        match custom_node.select_routes_for_forwarding(&bundle).await {
            Ok(routes) => {
                println!("  Selected {} routes for forwarding:", routes.len());
                for (i, route) in routes.iter().enumerate() {
                    println!(
                        "    {}. {} via {} (cost: {}, cla: {})",
                        i + 1,
                        route.next_hop,
                        route.next_hop,
                        route.cost,
                        route.cla_type
                    );
                }
            }
            Err(e) => {
                eprintln!("  ❌ Failed to select routes: {e}");
            }
        }
    }

    // Method 3: Show routing algorithm information
    println!("\n📋 Method 3: Routing algorithm information");
    let config = sdtn::config::Config::load()?;
    println!("  Current algorithm: {}", config.routing.algorithm);
    println!(
        "  Algorithm type: {:?}",
        config.get_routing_algorithm_type()
    );

    println!("\n✅ Real routing example completed!");
    println!("  This example shows how routing algorithms use routing tables");
    println!("  to select appropriate CLAs and next hops for bundle forwarding.");
    Ok(())
}
