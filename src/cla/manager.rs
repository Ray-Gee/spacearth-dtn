use crate::bpv7::bundle::Bundle;
use crate::cla::peer::ClaPeer;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait ConvergenceLayer: Send + Sync {
    fn address(&self) -> String;
    async fn activate(&self) -> anyhow::Result<()>;
}

// TODO: receive_callbackの責任分担を明確にする
// 現在ClaManagerとTcpClaListenerの両方でコールバックを保持している
// 理想的にはClaManagerが統一的にコールバックを管理すべき
pub struct ClaManager {
    state: Arc<RwLock<ClaState>>,
    receive_callback: Arc<dyn Fn(Bundle) + Send + Sync>,
}

#[derive(Default)]
struct ClaState {
    peers: Vec<Box<dyn ClaPeer>>,
}

impl ClaManager {
    pub fn new<F>(receive_callback: F) -> Self
    where
        F: Fn(Bundle) + Send + Sync + 'static,
    {
        Self {
            state: Arc::new(RwLock::new(ClaState::default())),
            receive_callback: Arc::new(receive_callback),
        }
    }

    /// Register a new peer (Box<dyn ClaPeer>)
    pub async fn register_peer(&self, peer: Box<dyn ClaPeer>) {
        let mut state = self.state.write().await;
        let peer_id = peer.get_peer_endpoint_id();
        if state
            .peers
            .iter()
            .any(|p| p.get_peer_endpoint_id() == peer_id)
        {
            println!("Peer already registered: {peer_id}");
            return;
        }
        if let Err(e) = peer.activate().await {
            println!("Failed to activate peer {peer_id}: {e}");
        }
        state.peers.push(peer);
    }

    pub fn notify_receive(&self, bundle: Bundle) {
        let cb = Arc::clone(&self.receive_callback);
        tokio::spawn(async move {
            cb(bundle);
        });
    }

    /// List all registered peers (regardless of reachability)
    pub async fn list_all_peers(&self) -> Vec<Box<dyn ClaPeer>> {
        let st = self.state.read().await;
        st.peers.iter().map(|p| p.clone_box()).collect()
    }

    /// List only reachable peers (filtered by is_reachable())
    pub async fn list_reachable_peers(&self) -> Vec<Box<dyn ClaPeer>> {
        let st = self.state.read().await;
        let mut reachable = Vec::new();
        for peer in &st.peers {
            if peer.is_reachable().await {
                reachable.push(peer.clone_box());
            }
        }
        reachable
    }

    /// Alias for list_reachable_peers (backward compatibility)
    pub async fn list_peers(&self) -> Vec<Box<dyn ClaPeer>> {
        self.list_reachable_peers().await
    }
}

impl Clone for ClaManager {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            receive_callback: Arc::clone(&self.receive_callback),
        }
    }
}
