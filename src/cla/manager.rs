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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{Bundle, PrimaryBlock};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex;

    // Mock ConvergenceLayer for testing
    #[derive(Debug)]
    struct MockCla {
        address: String,
        should_fail: bool,
        activation_counter: Arc<AtomicUsize>,
    }

    impl MockCla {
        fn new(address: &str) -> Self {
            Self {
                address: address.to_string(),
                should_fail: false,
                activation_counter: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn new_failing(address: &str) -> Self {
            Self {
                address: address.to_string(),
                should_fail: true,
                activation_counter: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn activation_count(&self) -> usize {
            self.activation_counter.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ConvergenceLayer for MockCla {
        fn address(&self) -> String {
            self.address.clone()
        }

        async fn activate(&self) -> anyhow::Result<()> {
            self.activation_counter.fetch_add(1, Ordering::SeqCst);

            if self.should_fail {
                return Err(anyhow::anyhow!("Mock activation failure"));
            }

            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(())
        }
    }

    fn create_test_bundle(source: &str, destination: &str) -> Bundle {
        Bundle {
            primary: PrimaryBlock {
                version: 7,
                source: source.to_string(),
                destination: destination.to_string(),
                report_to: "none".to_string(),
                creation_timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                lifetime: 3600,
            },
            payload: b"test payload".to_vec(),
        }
    }

    #[tokio::test]
    async fn test_cla_manager_new() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Test that manager was created successfully
        let active_clas = manager.list_active().await;
        assert!(active_clas.is_empty());
    }

    #[tokio::test]
    async fn test_register_single_cla() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let mock_cla = Arc::new(MockCla::new("test://127.0.0.1:8080"));

        manager.register(mock_cla).await;

        // Give some time for the registration to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let active_clas = manager.list_active().await;
        assert_eq!(active_clas.len(), 1);
        assert!(active_clas.contains(&"test://127.0.0.1:8080".to_string()));
    }

    #[tokio::test]
    async fn test_register_multiple_clas() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let cla1 = Arc::new(MockCla::new("test://127.0.0.1:8080"));
        let cla2 = Arc::new(MockCla::new("test://127.0.0.1:8081"));
        let cla3 = Arc::new(MockCla::new("test://127.0.0.1:8082"));

        manager.register(cla1).await;
        manager.register(cla2).await;
        manager.register(cla3).await;

        // Give some time for registrations to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let active_clas = manager.list_active().await;
        assert_eq!(active_clas.len(), 3);
        assert!(active_clas.contains(&"test://127.0.0.1:8080".to_string()));
        assert!(active_clas.contains(&"test://127.0.0.1:8081".to_string()));
        assert!(active_clas.contains(&"test://127.0.0.1:8082".to_string()));
    }

    #[tokio::test]
    async fn test_register_duplicate_cla() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let cla1 = Arc::new(MockCla::new("test://127.0.0.1:8080"));
        let cla2 = Arc::new(MockCla::new("test://127.0.0.1:8080")); // Same address

        manager.register(cla1).await;
        manager.register(cla2).await; // Should not register due to duplicate address

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let active_clas = manager.list_active().await;
        assert_eq!(active_clas.len(), 1);
    }

    #[tokio::test]
    async fn test_register_failing_cla() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let failing_cla = Arc::new(MockCla::new_failing("test://127.0.0.1:8080"));

        manager.register(failing_cla.clone()).await;

        // Give some time for activation to fail
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // CLA should still be registered even if activation failed
        let active_clas = manager.list_active().await;
        assert_eq!(active_clas.len(), 1);

        // Verify that activation was attempted
        assert_eq!(failing_cla.activation_count(), 1);
    }

    #[tokio::test]
    async fn test_notify_receive() {
        let received_bundles = Arc::new(Mutex::new(Vec::new()));
        let bundles_clone = Arc::clone(&received_bundles);

        let manager = ClaManager::new(move |bundle| {
            let bundles = Arc::clone(&bundles_clone);
            tokio::spawn(async move {
                let mut guard = bundles.lock().await;
                guard.push(bundle);
            });
        });

        let test_bundle = create_test_bundle("dtn://source", "dtn://dest");

        manager.notify_receive(test_bundle.clone());

        // Give some time for callback to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let received = received_bundles.lock().await;
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].primary.source, "dtn://source");
        assert_eq!(received[0].primary.destination, "dtn://dest");
    }

    #[tokio::test]
    async fn test_notify_receive_multiple_bundles() {
        let received_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&received_count);

        let manager = ClaManager::new(move |_bundle| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Send multiple bundles
        for i in 0..5 {
            let bundle =
                create_test_bundle(&format!("dtn://source{}", i), &format!("dtn://dest{}", i));
            manager.notify_receive(bundle);
        }

        // Give some time for all callbacks to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert_eq!(received_count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_manager_clone() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let manager1 = ClaManager::new(move |_bundle| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Clone the manager
        let manager2 = manager1.clone();

        // Register CLAs using both managers
        let cla1 = Arc::new(MockCla::new("test://127.0.0.1:8080"));
        let cla2 = Arc::new(MockCla::new("test://127.0.0.1:8081"));

        manager1.register(cla1).await;
        manager2.register(cla2).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Both managers should see the same state
        let active1 = manager1.list_active().await;
        let active2 = manager2.list_active().await;

        assert_eq!(active1.len(), 2);
        assert_eq!(active2.len(), 2);
        assert_eq!(active1, active2);
    }

    #[tokio::test]
    async fn test_list_active_empty() {
        let manager = ClaManager::new(|_bundle| {});

        let active_clas = manager.list_active().await;
        assert!(active_clas.is_empty());
    }

    #[tokio::test]
    async fn test_cla_state_default() {
        let state = ClaState::default();
        assert!(state.active_clas.is_empty());
    }

    #[test]
    fn test_mock_cla_address() {
        let mock_cla = MockCla::new("test://example.com:1234");
        assert_eq!(mock_cla.address(), "test://example.com:1234");
    }

    #[tokio::test]
    async fn test_mock_cla_activation_success() {
        let mock_cla = MockCla::new("test://example.com");
        let result = mock_cla.activate().await;
        assert!(result.is_ok());
        assert_eq!(mock_cla.activation_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_cla_activation_failure() {
        let mock_cla = MockCla::new_failing("test://example.com");
        let result = mock_cla.activate().await;
        assert!(result.is_err());
        assert_eq!(mock_cla.activation_count(), 1);
    }
}
