use crate::bpv7::bundle::*;
use crate::bpv7::EndpointId;
use crate::cla::manager::ClaManager;
use crate::cla::manager::ConvergenceLayer;
use crate::cla::peer::ClaPeer;
use crate::cla::TcpPeer;
use crate::config::{generate_creation_timestamp, Config};
use crate::consts::BUNDLES_DIR;
use crate::routing::algorithm::{RouteEntry, RoutingAlgorithm, RoutingConfig, RoutingTable};
use crate::store::bundle_descriptor::BundleDescriptor;
use crate::store::BundleStore;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

use super::BundleStatus;

/// DTN Node API for managing DTN bundles and network operations
pub struct DtnNode {
    store: BundleStore,
    store_path: String,
    routing_algorithm: Arc<TokioMutex<Box<dyn RoutingAlgorithm>>>,
    routing_table: Arc<Mutex<RoutingTable>>,
    cla_manager: Arc<ClaManager>,
}

impl DtnNode {
    /// Create a new DTN CLI instance with default bundle store path ("./bundles")
    pub fn new() -> anyhow::Result<Self> {
        // Priority: Env Var -> Config File -> Default
        let store_path = match std::env::var("SDTN_BUNDLE_PATH") {
            Ok(path) => path,
            Err(_) => match Config::load() {
                Ok(config) => config.storage.path,
                Err(_) => BUNDLES_DIR.to_string(),
            },
        };

        Self::with_store_path(&store_path)
    }

    /// Create a new DTN CLI instance with a custom bundle store path
    pub fn with_store_path(store_path: &str) -> anyhow::Result<Self> {
        let store = BundleStore::new(store_path)?;
        let config = Config::load()?;
        let routing_config = RoutingConfig::new(config.get_routing_algorithm_type());
        let routing_algorithm = Arc::new(TokioMutex::new(routing_config.create_algorithm()));
        let routing_table = Arc::new(Mutex::new(RoutingTable::new()));
        let cla_manager = Arc::new(ClaManager::new(|_bundle| {}));

        Ok(Self {
            store,
            store_path: store_path.to_string(),
            routing_algorithm,
            routing_table,
            cla_manager,
        })
    }

    /// Create a new DTN CLI instance with custom configuration
    pub fn with_config(store_path: Option<&str>) -> anyhow::Result<Self> {
        let path = store_path.unwrap_or(BUNDLES_DIR);
        Self::with_store_path(path)
    }

    /// Create a new DTN CLI instance with custom routing algorithm
    pub fn with_routing_algorithm(
        store_path: &str,
        routing_config: RoutingConfig,
    ) -> anyhow::Result<Self> {
        let store = BundleStore::new(store_path)?;
        let routing_algorithm = Arc::new(TokioMutex::new(routing_config.create_algorithm()));
        let routing_table = Arc::new(Mutex::new(RoutingTable::new()));
        let cla_manager = Arc::new(ClaManager::new(|_bundle| {}));

        Ok(Self {
            store,
            store_path: store_path.to_string(),
            routing_algorithm,
            routing_table,
            cla_manager,
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

    /// Get access to the routing table for advanced operations
    pub fn get_routing_table(&self) -> Arc<Mutex<RoutingTable>> {
        Arc::clone(&self.routing_table)
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
    pub async fn insert_bundle(&self, message: String) -> anyhow::Result<()> {
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
        let mut algorithm = self.routing_algorithm.lock().await;
        algorithm.notify_new_bundle(&descriptor);

        Ok(())
    }

    /// Select peers for forwarding a bundle (legacy method)
    pub async fn select_peers_for_forwarding(
        &self,
        bundle: &Bundle,
    ) -> anyhow::Result<Vec<Box<dyn ClaPeer>>> {
        let descriptor = BundleDescriptor::new(bundle.clone());

        // Get reachable peers from CLA manager
        let peers = self.cla_manager.list_reachable_peers().await;

        let algorithm = self.routing_algorithm.lock().await;
        let selected_refs = algorithm.select_peers_for_forwarding(&descriptor, &peers);

        // Convert references back to owned boxes (this is a bit awkward, but necessary for the trait)
        let result = selected_refs
            .into_iter()
            .map(|peer_ref| {
                let eid = peer_ref.get_peer_endpoint_id();
                let address = peer_ref.get_connection_address();
                Box::new(TcpPeer::new(eid, address)) as Box<dyn ClaPeer>
            })
            .collect();

        Ok(result)
    }

    /// Select routes for forwarding a bundle (new method using routing table)
    pub async fn select_routes_for_forwarding(
        &self,
        bundle: &Bundle,
    ) -> anyhow::Result<Vec<RouteEntry>> {
        let descriptor = BundleDescriptor::new(bundle.clone());

        let algorithm = self.routing_algorithm.lock().await;
        if let Ok(table) = self.routing_table.lock() {
            let routes = algorithm.select_routes_for_forwarding(&descriptor, &table);
            Ok(routes)
        } else {
            anyhow::bail!("Failed to lock routing table")
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
            bind_addr: bind_addr.clone(),
            receive_callback: Arc::new(move |bundle| {
                // バンドル受信時の保存処理
                if let Ok(store) = BundleStore::new(&store_path) {
                    let _ = store.insert(&bundle);
                }
            }),
        });

        // CLAマネージャにピア登録（必要なら）
        let manager = ClaManager::new(|bundle| {
            println!("📥 Received: {bundle:?}");
        });
        let peer: Box<dyn ClaPeer> =
            Box::new(TcpPeer::new(EndpointId::from("dtn://listener"), bind_addr));
        manager.register_peer(peer).await;

        // CLAリスナーを起動
        cla.activate().await?;

        Ok(())
    }

    /// Start a TCP dialer daemon
    pub async fn start_tcp_dialer(&self, target_addr: String) -> anyhow::Result<()> {
        let manager = ClaManager::new(|bundle| {
            println!("📤 Should not receive here (Dialer): {bundle:?}");
        });

        let peer: Box<dyn ClaPeer> =
            Box::new(TcpPeer::new(EndpointId::from("dtn://dialer"), target_addr));
        manager.register_peer(peer).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Ok(())
    }

    /// Select peers for forwarding a bundle with connectivity check (async version)
    pub async fn select_peers_for_forwarding_async(
        &self,
        bundle: &Bundle,
    ) -> anyhow::Result<Vec<Box<dyn ClaPeer>>> {
        let descriptor = BundleDescriptor::new(bundle.clone());

        // Get reachable peers from CLA manager
        let peers = self.cla_manager.list_reachable_peers().await;

        let algorithm = self.routing_algorithm.lock().await;
        let selected_refs = algorithm
            .select_peers_for_forwarding_async(&descriptor, &peers)
            .await;

        // Convert references back to owned boxes
        let result = selected_refs
            .into_iter()
            .map(|peer_ref| {
                let eid = peer_ref.get_peer_endpoint_id();
                let address = peer_ref.get_connection_address();
                Box::new(TcpPeer::new(eid, address)) as Box<dyn ClaPeer>
            })
            .collect();

        Ok(result)
    }
}

/// Default implementation for DtnNode
impl Default for DtnNode {
    fn default() -> Self {
        Self::new().expect("Failed to create default DtnNode")
    }
}
