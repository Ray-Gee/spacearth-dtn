use crate::bpv7::bundle::*;
use crate::cla::manager::ClaManager;
use crate::config::{generate_creation_timestamp, Config};
use crate::routing::algorithm::{
    ConvergenceSender, RouteEntry, RoutingAlgorithm, RoutingConfig, RoutingTable, TcpSender,
};
use crate::store::bundle_descriptor::BundleDescriptor;
use crate::store::BundleStore;
use std::sync::{Arc, Mutex};

use super::BundleStatus;

/// DTN Node API for managing DTN bundles and network operations
pub struct DtnNode {
    store: BundleStore,
    store_path: String,
    routing_algorithm: Arc<Mutex<Box<dyn RoutingAlgorithm>>>,
    routing_table: Arc<Mutex<RoutingTable>>,
}

impl DtnNode {
    /// Create a new DTN CLI instance with default bundle store path ("./bundles")
    pub fn new() -> anyhow::Result<Self> {
        Self::with_store_path("./bundles")
    }

    /// Create a new DTN CLI instance with a custom bundle store path
    pub fn with_store_path(store_path: &str) -> anyhow::Result<Self> {
        let store = BundleStore::new(store_path)?;
        let config = Config::load()?;
        let routing_config = RoutingConfig::new(config.get_routing_algorithm_type());
        let routing_algorithm = Arc::new(Mutex::new(routing_config.create_algorithm()));
        let routing_table = Arc::new(Mutex::new(RoutingTable::new()));

        Ok(Self {
            store,
            store_path: store_path.to_string(),
            routing_algorithm,
            routing_table,
        })
    }

    /// Create a new DTN CLI instance with custom configuration
    pub fn with_config(store_path: Option<&str>) -> anyhow::Result<Self> {
        let path = store_path.unwrap_or("./bundles");
        Self::with_store_path(path)
    }

    /// Create a new DTN CLI instance with custom routing algorithm
    pub fn with_routing_algorithm(
        store_path: &str,
        routing_config: RoutingConfig,
    ) -> anyhow::Result<Self> {
        let store = BundleStore::new(store_path)?;
        let routing_algorithm = Arc::new(Mutex::new(routing_config.create_algorithm()));
        let routing_table = Arc::new(Mutex::new(RoutingTable::new()));

        Ok(Self {
            store,
            store_path: store_path.to_string(),
            routing_algorithm,
            routing_table,
        })
    }

    /// Add a route to the routing table
    pub fn add_route(&self, entry: RouteEntry) -> anyhow::Result<()> {
        if let Ok(mut table) = self.routing_table.lock() {
            table.add_route(entry);
            Ok(())
        } else {
            anyhow::bail!("Failed to lock routing table")
        }
    }

    /// Get all routes from the routing table
    pub fn get_all_routes(&self) -> anyhow::Result<Vec<RouteEntry>> {
        if let Ok(table) = self.routing_table.lock() {
            Ok(table.get_all_routes().into_iter().cloned().collect())
        } else {
            anyhow::bail!("Failed to lock routing table")
        }
    }

    /// Find the best route for a destination
    pub fn find_best_route(
        &self,
        destination: &crate::bpv7::EndpointId,
    ) -> anyhow::Result<Option<RouteEntry>> {
        if let Ok(table) = self.routing_table.lock() {
            Ok(table.find_best_route(destination).cloned())
        } else {
            anyhow::bail!("Failed to lock routing table")
        }
    }

    /// Insert a new bundle with the given message
    pub fn insert_bundle(&self, message: String) -> anyhow::Result<()> {
        #[cfg(test)]
        let config = {
            // In tests, use a slightly different timestamp each time to avoid duplicates
            std::thread::sleep(std::time::Duration::from_millis(1));
            Config::test_config()
        };
        #[cfg(not(test))]
        let config = Config::load()?;

        let bundle = Bundle {
            primary: PrimaryBlock {
                version: config.bundle.version,
                destination: config.endpoints.destination,
                source: config.endpoints.source,
                report_to: config.endpoints.report_to,
                creation_timestamp: generate_creation_timestamp(),
                lifetime: config.bundle.lifetime,
            },
            payload: message.into_bytes(),
        };

        self.store.insert(&bundle)?;

        // Notify routing algorithm about new bundle
        let descriptor = BundleDescriptor::new(bundle);
        if let Ok(mut algorithm) = self.routing_algorithm.lock() {
            algorithm.notify_new_bundle(&descriptor);
        }

        Ok(())
    }

    /// Select peers for forwarding a bundle (legacy method)
    pub fn select_peers_for_forwarding(
        &self,
        bundle: &Bundle,
    ) -> anyhow::Result<Vec<Box<dyn ConvergenceSender>>> {
        let descriptor = BundleDescriptor::new(bundle.clone());

        // For now, create some dummy senders for demonstration
        // In a real implementation, this would come from the CLA manager
        let senders: Vec<Box<dyn ConvergenceSender>> = vec![
            Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer1"))),
            Box::new(TcpSender::new(crate::bpv7::EndpointId::from("dtn://peer2"))),
        ];

        if let Ok(algorithm) = self.routing_algorithm.lock() {
            let selected_refs = algorithm.select_peers_for_forwarding(&descriptor, &senders);

            // Convert references back to owned boxes (this is a bit awkward, but necessary for the trait)
            let result = selected_refs
                .into_iter()
                .map(|sender_ref| {
                    let eid = sender_ref.get_peer_endpoint_id();
                    Box::new(TcpSender::new(eid)) as Box<dyn ConvergenceSender>
                })
                .collect();

            Ok(result)
        } else {
            anyhow::bail!("Failed to lock routing algorithm")
        }
    }

    /// Select routes for forwarding a bundle (new method using routing table)
    pub fn select_routes_for_forwarding(&self, bundle: &Bundle) -> anyhow::Result<Vec<RouteEntry>> {
        let descriptor = BundleDescriptor::new(bundle.clone());

        if let Ok(algorithm) = self.routing_algorithm.lock() {
            if let Ok(table) = self.routing_table.lock() {
                let routes = algorithm.select_routes_for_forwarding(&descriptor, &table);
                Ok(routes)
            } else {
                anyhow::bail!("Failed to lock routing table")
            }
        } else {
            anyhow::bail!("Failed to lock routing algorithm")
        }
    }

    /// List all bundle IDs
    pub fn list_bundles(&self) -> anyhow::Result<Vec<String>> {
        self.store.list()
    }

    /// Show bundle details by partial ID
    pub fn show_bundle(&self, partial_id: &str) -> anyhow::Result<Bundle> {
        self.store.load_by_partial_id(partial_id)
    }

    /// Get bundle status information
    pub fn get_bundle_status(&self, partial_id: Option<&str>) -> anyhow::Result<BundleStatus> {
        match partial_id {
            Some(id) => {
                let bundle = self.store.load_by_partial_id(id)?;
                Ok(BundleStatus::Single {
                    id: id.to_string(),
                    bundle,
                })
            }
            None => {
                let bundles = self.store.list()?;
                let mut active_count = 0;
                let mut expired_count = 0;

                for id in &bundles {
                    if let Ok(bundle) = self.store.load_by_partial_id(id) {
                        if bundle.is_expired() {
                            expired_count += 1;
                        } else {
                            active_count += 1;
                        }
                    }
                }

                Ok(BundleStatus::Summary {
                    active: active_count,
                    expired: expired_count,
                    total: active_count + expired_count,
                })
            }
        }
    }

    /// Clean up expired bundles
    pub fn cleanup_expired(&self) -> anyhow::Result<()> {
        self.store.cleanup_expired()
    }

    /// Start a TCP listener daemon
    pub async fn start_tcp_listener(&self, bind_addr: String) -> anyhow::Result<()> {
        let store_path = self.store_path.clone();
        let cla = Arc::new(crate::cla::TcpClaListener {
            bind_addr,
            receive_callback: Arc::new(move |bundle| {
                if let Err(e) = (|| -> anyhow::Result<()> {
                    let store = BundleStore::new(&store_path)?;
                    store.insert(&bundle)?;
                    Ok(())
                })() {
                    eprintln!("❌ Failed to insert bundle: {e}");
                }
            }),
        });

        let manager = ClaManager::new(|bundle| {
            println!("📥 Received: {:?}", bundle);
        });

        manager.register(cla).await;
        futures::future::pending::<()>().await;
        Ok(())
    }

    /// Start a TCP dialer daemon
    pub async fn start_tcp_dialer(&self, target_addr: String) -> anyhow::Result<()> {
        let cla = Arc::new(crate::cla::TcpClaDialer { target_addr });
        let manager = ClaManager::new(|bundle| {
            println!("📤 Should not receive here (Dialer): {:?}", bundle);
        });

        manager.register(cla).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Ok(())
    }
}

/// Default implementation for DtnNode
impl Default for DtnNode {
    fn default() -> Self {
        Self::new().expect("Failed to create default DtnNode")
    }
}
