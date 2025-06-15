use crate::bundle::Bundle;
use async_trait::async_trait;
use std::collections::HashSet;
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

#[derive(Debug, Default)]
struct ClaState {
    active_clas: HashSet<String>,
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

    pub async fn register(&self, cla: Arc<dyn ConvergenceLayer>) {
        let address = cla.address();
        {
            let mut state = self.state.write().await;
            if !state.active_clas.insert(address.clone()) {
                println!("CLA already registered: {}", address);
                return;
            }
        }

        tokio::spawn(async move {
            match cla.activate().await {
                Ok(()) => println!("CLA activated: {address}"),
                Err(e) => eprintln!("Failed to activate CLA ({address}): {e:?}"),
            }
        });
    }

    pub fn notify_receive(&self, bundle: Bundle) {
        let cb = Arc::clone(&self.receive_callback);
        tokio::spawn(async move {
            cb(bundle);
        });
    }

    pub async fn list_active(&self) -> Vec<String> {
        let st = self.state.read().await;
        st.active_clas.iter().cloned().collect()
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
