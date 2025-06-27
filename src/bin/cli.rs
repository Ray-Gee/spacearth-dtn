use clap::Parser;
use sdtn::api::DtnNode;
use sdtn::bpv7::EndpointId;
use sdtn::routing::algorithm::RouteEntry;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Insert {
        #[clap(short, long)]
        message: String,
    },
    List,
    Show {
        #[clap(short, long)]
        id: String,
    },
    Status {
        /// Show detailed status including expiration
        #[clap(short, long)]
        id: Option<String>,
    },
    Receive,
    Daemon {
        #[clap(subcommand)]
        cmd: DaemonCmd,
    },
    Cleanup,
    Route {
        #[clap(subcommand)]
        cmd: RouteCmd,
    },
}

#[derive(Parser)]
pub enum DaemonCmd {
    Listener {
        #[clap(long)]
        addr: String,
    },
    Dialer {
        #[clap(long)]
        addr: String,
    },
}

#[derive(Parser)]
pub enum RouteCmd {
    /// Test routing algorithm with a specific bundle
    Test {
        #[clap(short, long)]
        id: String,
    },
    /// Show current routing algorithm
    Show,
    /// Set routing algorithm
    Set {
        #[clap(short, long)]
        algorithm: String,
    },
    /// Show routing table
    Table,
    /// Add route to routing table
    Add {
        #[clap(long)]
        destination: String,
        #[clap(long)]
        next_hop: String,
        #[clap(long)]
        cla_type: String,
        #[clap(long, default_value = "10")]
        cost: u32,
    },
    /// Test routing with routing table
    TestTable {
        #[clap(short, long)]
        id: String,
    },
}

// Split command handling into separate functions for better testability
pub async fn handle_insert_command(node: &DtnNode, message: String) -> anyhow::Result<()> {
    println!("📦 Inserting bundle: {message}");
    node.insert_bundle(message).await?;
    println!("✅ Bundle inserted successfully!");
    Ok(())
}

pub fn handle_list_command(node: &DtnNode) -> anyhow::Result<()> {
    let bundles = node.list_bundles()?;
    if bundles.is_empty() {
        println!("📋 No bundles found");
    } else {
        println!("📋 Found {} bundles:", bundles.len());
        for id in bundles {
            println!("  {id}");
        }
    }
    Ok(())
}

pub fn handle_show_command(node: &DtnNode, id: String) -> anyhow::Result<()> {
    let bundle = node.show_bundle(&id)?;
    println!("📄 Bundle Details:");
    println!("  Source: {}", bundle.primary.source);
    println!("  Destination: {}", bundle.primary.destination);
    println!("  Creation Time: {}", bundle.primary.creation_timestamp);
    println!("  Lifetime: {} seconds", bundle.primary.lifetime);
    println!("  Expired: {}", bundle.is_expired());
    println!("  Message: {}", String::from_utf8_lossy(&bundle.payload));
    Ok(())
}

pub fn handle_status_command(node: &DtnNode, id: Option<String>) -> anyhow::Result<()> {
    match id {
        Some(bundle_id) => {
            let bundle = node.show_bundle(&bundle_id)?;

            println!("📄 Bundle Status: {bundle_id}");
            println!("  Source: {}", bundle.primary.source);
            println!("  Destination: {}", bundle.primary.destination);
            println!("  Creation Time: {}", bundle.primary.creation_timestamp);
            println!("  Lifetime: {} seconds", bundle.primary.lifetime);
            println!(
                "  Status: {}",
                if bundle.is_expired() {
                    "⏰ EXPIRED"
                } else {
                    "✅ ACTIVE"
                }
            );
            println!("  Message: {}", String::from_utf8_lossy(&bundle.payload));
        }
        None => {
            // Show status of all bundles
            let status = node.get_bundle_status(None)?;
            match status {
                sdtn::api::BundleStatus::Summary {
                    active,
                    expired,
                    total,
                } => {
                    println!("📊 Bundle Status Summary:");
                    println!("  ✅ Active: {active}");
                    println!("  ⏰ Expired: {expired}");
                    println!("  📦 Total: {total}");
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}

pub fn handle_cleanup_command(node: &DtnNode) -> anyhow::Result<()> {
    node.cleanup_expired()?;
    Ok(())
}

pub async fn handle_route_test_command(node: &DtnNode, id: String) -> anyhow::Result<()> {
    let bundle = node.show_bundle(&id)?;
    println!("🧭 Testing routing for bundle: {id}");
    println!("  Source: {}", bundle.primary.source);
    println!("  Destination: {}", bundle.primary.destination);

    match node.select_peers_for_forwarding(&bundle).await {
        Ok(peers) => {
            println!("  Selected {} peers for forwarding:", peers.len());
            for (i, peer) in peers.iter().enumerate() {
                println!("    {}. {}", i + 1, peer.get_peer_endpoint_id());
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to select peers: {e}");
        }
    }
    Ok(())
}

pub fn handle_route_show_command() -> anyhow::Result<()> {
    println!("🧭 Current routing algorithm:");
    // For now, we'll show the algorithm type from config
    let config = sdtn::config::Config::load()?;
    println!("  Algorithm: {}", config.routing.algorithm);
    Ok(())
}

pub fn handle_route_set_command(algorithm: String) -> anyhow::Result<()> {
    println!("🧭 Setting routing algorithm to: {algorithm}");
    println!("⚠️  This feature requires restarting the application");
    println!("   Update config/default.toml or set DTN_ROUTING_ALGORITHM environment variable");
    Ok(())
}

pub fn handle_route_table_command(node: &DtnNode) -> anyhow::Result<()> {
    println!("🧭 Routing Table:");
    match node.get_all_routes() {
        Ok(routes) => {
            if routes.is_empty() {
                println!("  No routes configured");
            } else {
                for (i, route) in routes.iter().enumerate() {
                    println!(
                        "  {}. {} -> {} via {} (cost: {}, cla: {}, active: {})",
                        i + 1,
                        route.destination,
                        route.next_hop,
                        route.next_hop,
                        route.cost,
                        route.cla_type,
                        route.is_active
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get routing table: {e}");
        }
    }
    Ok(())
}

pub fn handle_route_add_command(
    node: &DtnNode,
    destination: String,
    next_hop: String,
    cla_type: String,
    cost: u32,
) -> anyhow::Result<()> {
    println!("🧭 Adding route to routing table:");
    println!("  Destination: {destination}");
    println!("  Next hop: {next_hop}");
    println!("  CLA type: {cla_type}");
    println!("  Cost: {cost}");

    let entry = RouteEntry {
        destination: EndpointId::from(&destination),
        next_hop: EndpointId::from(&next_hop),
        cla_type,
        cost,
        is_active: true,
    };

    match node.add_route(entry) {
        Ok(()) => println!("✅ Route added successfully!"),
        Err(e) => eprintln!("❌ Failed to add route: {e}"),
    }
    Ok(())
}

pub async fn handle_route_test_table_command(node: &DtnNode, id: String) -> anyhow::Result<()> {
    let bundle = node.show_bundle(&id)?;
    println!("🧭 Testing routing table for bundle: {id}");
    println!("  Source: {}", bundle.primary.source);
    println!("  Destination: {}", bundle.primary.destination);

    // Test routing with routing table
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
            eprintln!("❌ Failed to select routes: {e}");
        }
    }

    // Test finding best route
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
    Ok(())
}

pub async fn handle_daemon_listener_command(node: &DtnNode, addr: String) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            node.start_tcp_listener(addr).await.unwrap();
        });
    Ok(())
}

pub async fn handle_daemon_dialer_command(node: &DtnNode, addr: String) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            node.start_tcp_dialer(addr).await.unwrap();
        });
    Ok(())
}

pub async fn execute_command(node: &DtnNode, cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Insert { message } => handle_insert_command(node, message).await,
        Command::List => handle_list_command(node),
        Command::Show { id } => handle_show_command(node, id),
        Command::Status { id } => handle_status_command(node, id),
        Command::Receive => {
            todo!();
        }
        Command::Daemon { cmd } => match cmd {
            DaemonCmd::Listener { addr } => {
                tokio::runtime::Runtime::new()?
                    .block_on(async { handle_daemon_listener_command(node, addr).await })?;
                Ok(())
            }
            DaemonCmd::Dialer { addr } => {
                tokio::runtime::Runtime::new()?
                    .block_on(async { handle_daemon_dialer_command(node, addr).await })?;
                Ok(())
            }
        },
        Command::Cleanup => handle_cleanup_command(node),
        Command::Route { cmd } => match cmd {
            RouteCmd::Test { id } => handle_route_test_command(node, id).await,
            RouteCmd::Show => handle_route_show_command(),
            RouteCmd::Set { algorithm } => handle_route_set_command(algorithm),
            RouteCmd::Table => handle_route_table_command(node),
            RouteCmd::Add {
                destination,
                next_hop,
                cla_type,
                cost,
            } => handle_route_add_command(node, destination, next_hop, cla_type, cost),
            RouteCmd::TestTable { id } => handle_route_test_table_command(node, id).await,
        },
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let node = DtnNode::new()?;
    tokio::runtime::Runtime::new()?.block_on(async { execute_command(&node, opts.cmd).await })
}
