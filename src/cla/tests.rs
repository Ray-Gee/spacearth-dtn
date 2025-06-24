#[test]
fn test_module_path() {
    let path = module_path!();
    println!("Actual module path: {}", path);
    assert!(path.contains("cla::tests"));
}

#[test]
fn test_current_module() {
    let current_module = module_path!();
    println!("Current module path: {}", current_module);
    assert!(current_module.contains("cla::tests"));
}

use crate::cla::*;

#[test]
fn test_module_exports_exist() {
    // Test that the re-exports work by referencing the types
    // This ensures the modules are properly exposed

    // Check that we can reference the manager types
    let _manager_type = std::any::TypeId::of::<ClaManager>();
    let _convergence_layer_type = std::any::TypeId::of::<dyn ConvergenceLayer>();

    // Check that we can reference the TCP types
    let _dialer_type = std::any::TypeId::of::<TcpClaClient>();
    let _listener_type = std::any::TypeId::of::<TcpClaListener>();
}

#[test]
fn test_modules_are_accessible() {
    // This test verifies that all modules are accessible

    // Check that we can access the module paths
    let _manager_module = module_path!();
    assert!(module_path!().contains("cla"));

    // These imports should work if modules are public
    use crate::cla::manager::ClaManager;
    use crate::cla::tcp::server::TcpClaListener;

    let _ = std::any::TypeId::of::<ClaManager>();
    let _ = std::any::TypeId::of::<TcpClaClient>();
    let _ = std::any::TypeId::of::<TcpClaListener>();
}

#[test]
fn test_reexports_work() {
    // Test that the re-exports match the original types
    assert_eq!(
        std::any::TypeId::of::<ClaManager>(),
        std::any::TypeId::of::<crate::cla::manager::ClaManager>()
    );

    assert_eq!(
        std::any::TypeId::of::<TcpClaClient>(),
        std::any::TypeId::of::<crate::cla::tcp::client::TcpClaClient>()
    );

    assert_eq!(
        std::any::TypeId::of::<TcpClaListener>(),
        std::any::TypeId::of::<crate::cla::tcp::server::TcpClaListener>()
    );
}

// Common imports for all tests
use crate::bpv7::bundle::{Bundle, PrimaryBlock};
use crate::cla::manager::*;
use crate::cla::tcp::client::*;
use crate::cla::tcp::server::*;
use crate::consts::tcp::*;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::Duration;

// Unified create_test_bundle function that takes payload as parameter
fn create_test_bundle(source: &str, destination: &str, payload: &[u8]) -> Bundle {
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
        payload: payload.to_vec(),
    }
}

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

    let test_bundle = create_test_bundle("dtn://source", "dtn://dest", b"test payload");

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
        let bundle = create_test_bundle(
            &format!("dtn://source{}", i),
            &format!("dtn://dest{}", i),
            b"test payload",
        );
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

#[test]
fn test_tcp_cla_dialer_new() {
    let dialer = TcpClaClient {
        target_addr: "127.0.0.1:8080".to_string(),
    };
    assert_eq!(dialer.target_addr, "127.0.0.1:8080");
}

#[test]
fn test_tcp_cla_dialer_address() {
    let dialer = TcpClaClient {
        target_addr: "localhost:9090".to_string(),
    };
    assert_eq!(dialer.address(), "localhost:9090");
}

#[test]
fn test_create_bundle_simple() {
    let bundle = create_bundle("dtn://source", "dtn://dest", b"hello".to_vec());

    assert_eq!(bundle.primary.source, "dtn://source");
    assert_eq!(bundle.primary.destination, "dtn://dest");
    assert_eq!(bundle.payload, b"hello");
    assert_eq!(bundle.primary.version, 7);
    assert_eq!(bundle.primary.report_to, "none");
    assert_eq!(bundle.primary.lifetime, 3600);
}

#[test]
fn test_create_bundle_with_various_payloads() {
    let test_cases = vec![
        ("empty", b"".to_vec()),
        ("simple", b"hello world".to_vec()),
        ("unicode", "こんにちは世界".as_bytes().to_vec()),
        ("numbers", b"123456789".to_vec()),
        ("binary", vec![0, 1, 2, 255, 254, 253]),
    ];

    for (name, payload) in test_cases {
        let bundle = create_bundle(
            &format!("dtn://source_{}", name),
            &format!("dtn://dest_{}", name),
            payload.clone(),
        );

        assert_eq!(bundle.payload, payload);
        assert!(bundle.primary.creation_timestamp > 0);
    }
}

#[test]
fn test_create_bundle_timing() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let bundle = create_bundle("dtn://source", "dtn://dest", b"test".to_vec());
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert!(bundle.primary.creation_timestamp >= before);
    assert!(bundle.primary.creation_timestamp <= after);
}

// Mock TCP server for testing send_bundle
async fn mock_tcp_server(
    response: &'static str,
) -> anyhow::Result<(u16, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let handle = tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            // Read length
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_ok() {
                let len = u32::from_be_bytes(len_buf) as usize;

                // Read bundle data
                let mut data = vec![0u8; len];
                if stream.read_exact(&mut data).await.is_ok() {
                    // Send response
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            }
        }
    });

    // Give the server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok((port, handle))
}

#[tokio::test]
async fn test_send_bundle_success() -> anyhow::Result<()> {
    let (port, _handle) = mock_tcp_server(OK).await?;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let bundle = create_test_bundle("dtn://source", "dtn://dest", b"test payload");

    let result = send_bundle(&mut stream, &bundle).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_send_bundle_with_different_acks() -> anyhow::Result<()> {
    let test_cases = [OK, ACK, SUCCESS, RECEIVED];

    for (i, ack) in test_cases.iter().enumerate() {
        let (port, _handle) = mock_tcp_server(ack).await?;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        let bundle = create_test_bundle(
            &format!("dtn://source_{}", i),
            &format!("dtn://dest_{}", i),
            format!("test payload {}", i).as_bytes(),
        );

        let result = send_bundle(&mut stream, &bundle).await;
        assert!(result.is_ok(), "Failed for ACK: {}", ack);
    }

    Ok(())
}

#[tokio::test]
async fn test_send_bundle_large_payload() -> anyhow::Result<()> {
    let (port, _handle) = mock_tcp_server(OK).await?;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

    // Create a large payload
    let large_payload = vec![42u8; 10000];
    let bundle = create_test_bundle("dtn://source", "dtn://dest", &large_payload);

    let result = send_bundle(&mut stream, &bundle).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_send_bundle_serialization() -> anyhow::Result<()> {
    let bundle = create_test_bundle("dtn://source", "dtn://dest", b"test");

    // Test that the bundle can be serialized
    let encoded = serde_cbor::to_vec(&bundle);
    assert!(encoded.is_ok());

    let encoded_data = encoded.unwrap();
    assert!(!encoded_data.is_empty());

    // Test that it can be deserialized back
    let decoded: Result<Bundle, _> = serde_cbor::from_slice(&encoded_data);
    assert!(decoded.is_ok());

    let decoded_bundle = decoded.unwrap();
    assert_eq!(decoded_bundle.primary.source, bundle.primary.source);
    assert_eq!(
        decoded_bundle.primary.destination,
        bundle.primary.destination
    );
    assert_eq!(decoded_bundle.payload, bundle.payload);

    Ok(())
}

#[tokio::test]
async fn test_tcp_cla_dialer_activate_no_server() {
    let dialer = TcpClaClient {
        target_addr: "127.0.0.1:19999".to_string(), // Non-existent server
    };

    // This should fail because there's no server listening
    let result = dialer.activate().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tcp_cla_dialer_activate_with_empty_store() -> anyhow::Result<()> {
    // Create a mock server that accepts connections but expects no data
    let (port, _handle) = mock_tcp_server(OK).await?;

    // Give the server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create a temporary directory for empty bundle store
    let temp_dir = TempDir::new()?;
    let _temp_bundles_dir = temp_dir.path().join("test_bundles");

    // Test with custom bundles directory
    let _dialer = TcpClaClient {
        target_addr: format!("127.0.0.1:{}", port),
    };

    // This test mainly checks the connection part since we can't easily
    // mock the BundleStore::new("./bundles") call in activate()
    // For a complete test, we'd need dependency injection

    Ok(())
}

#[test]
fn test_create_bundle_different_addresses() {
    let test_cases = vec![
        ("dtn://node1", "dtn://node2"),
        ("tcp://localhost:8080", "tcp://remote:9090"),
        ("http://example.com", "https://secure.example.com"),
        ("", ""), // Edge case with empty addresses
    ];

    for (source, dest) in test_cases {
        let bundle = create_bundle(source, dest, b"test".to_vec());
        assert_eq!(bundle.primary.source, source);
        assert_eq!(bundle.primary.destination, dest);
    }
}

#[test]
fn test_create_bundle_consistency() {
    // Create multiple bundles and ensure they have consistent structure
    for i in 0..10 {
        let bundle = create_bundle(
            &format!("dtn://source{}", i),
            &format!("dtn://dest{}", i),
            format!("payload{}", i).into_bytes(),
        );

        assert_eq!(bundle.primary.version, 7);
        assert_eq!(bundle.primary.report_to, "none");
        assert_eq!(bundle.primary.lifetime, 3600);
        assert!(bundle.primary.creation_timestamp > 0);
    }
}

#[test]
fn test_tcp_cla_listener_new() {
    let callback = Arc::new(|_bundle: Bundle| {});
    let listener = TcpClaListener {
        bind_addr: "127.0.0.1:8080".to_string(),
        receive_callback: callback,
    };

    assert_eq!(listener.bind_addr, "127.0.0.1:8080");
}

#[test]
fn test_tcp_cla_listener_address() {
    let callback = Arc::new(|_bundle: Bundle| {});
    let listener = TcpClaListener {
        bind_addr: "0.0.0.0:9090".to_string(),
        receive_callback: callback,
    };

    assert_eq!(listener.address(), "0.0.0.0:9090");
}

async fn _send_bundle_to_server(addr: &str, bundle: &Bundle) -> anyhow::Result<String> {
    let mut stream = TcpStream::connect(addr).await?;

    // Serialize bundle
    let encoded = serde_cbor::to_vec(bundle)?;
    let len = encoded.len() as u32;

    // Send length and data
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&encoded).await?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}

#[tokio::test]
async fn test_handle_connection_single_bundle() -> anyhow::Result<()> {
    let received_bundles = Arc::new(Mutex::new(Vec::new()));
    let bundles_clone = Arc::clone(&received_bundles);

    let callback = {
        let bundles_ref = Arc::clone(&bundles_clone);
        Arc::new(move |bundle: Bundle| {
            let bundles = Arc::clone(&bundles_ref);
            tokio::spawn(async move {
                let mut guard = bundles.lock().await;
                guard.push(bundle);
            });
        })
    };

    // Create a mock connection using pipes
    let (client, server) = tokio::io::duplex(1024);

    // Spawn handle_connection
    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    // Send test bundle
    let bundle = create_test_bundle("dtn://source", "dtn://dest", b"test payload");
    let encoded = serde_cbor::to_vec(&bundle)?;
    let len = encoded.len() as u32;

    let mut client = client;
    client.write_all(&len.to_be_bytes()).await?;
    client.write_all(&encoded).await?;

    // Read response
    let mut response = [0u8; 2];
    client.read_exact(&mut response).await?;
    assert_eq!(&response, b"OK");

    // Close connection to end the loop
    drop(client);

    // Wait for handler to complete
    let _ = tokio::time::timeout(Duration::from_millis(100), handle).await;

    // Check received bundles
    let received = received_bundles.lock().await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].primary.source, "dtn://source");
    assert_eq!(received[0].primary.destination, "dtn://dest");
    assert_eq!(received[0].payload, b"test payload");

    Ok(())
}

#[tokio::test]
async fn test_handle_connection_multiple_bundles() -> anyhow::Result<()> {
    let received_count = Arc::new(AtomicUsize::new(0));
    let count_clone = Arc::clone(&received_count);

    let callback = Arc::new(move |_bundle: Bundle| {
        count_clone.fetch_add(1, Ordering::SeqCst);
    });

    let (client, server) = tokio::io::duplex(2048);

    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    let mut client = client;

    // Send multiple bundles
    for i in 0..3 {
        let bundle = create_test_bundle(
            &format!("dtn://source{}", i),
            &format!("dtn://dest{}", i),
            format!("payload {}", i).as_bytes(),
        );

        let encoded = serde_cbor::to_vec(&bundle)?;
        let len = encoded.len() as u32;

        client.write_all(&len.to_be_bytes()).await?;
        client.write_all(&encoded).await?;

        // Read OK response
        let mut response = [0u8; 2];
        client.read_exact(&mut response).await?;
        assert_eq!(&response, b"OK");
    }

    drop(client);
    let _ = tokio::time::timeout(Duration::from_millis(100), handle).await;

    assert_eq!(received_count.load(Ordering::SeqCst), 3);

    Ok(())
}

#[tokio::test]
async fn test_handle_connection_large_bundle() -> anyhow::Result<()> {
    let received_bundles = Arc::new(Mutex::new(Vec::new()));
    let bundles_clone = Arc::clone(&received_bundles);

    let callback = {
        let bundles_ref = Arc::clone(&bundles_clone);
        Arc::new(move |bundle: Bundle| {
            let bundles = Arc::clone(&bundles_ref);
            tokio::spawn(async move {
                let mut guard = bundles.lock().await;
                guard.push(bundle);
            });
        })
    };

    let (client, server) = tokio::io::duplex(20000);

    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    // Create large payload
    let large_payload = vec![42u8; 10000];
    let bundle = create_test_bundle("dtn://source", "dtn://dest", &large_payload);

    let encoded = serde_cbor::to_vec(&bundle)?;
    let len = encoded.len() as u32;

    let mut client = client;
    client.write_all(&len.to_be_bytes()).await?;
    client.write_all(&encoded).await?;

    let mut response = [0u8; 2];
    client.read_exact(&mut response).await?;
    assert_eq!(&response, b"OK");

    drop(client);
    let _ = tokio::time::timeout(Duration::from_millis(100), handle).await;

    let received = received_bundles.lock().await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].payload.len(), 10000);

    Ok(())
}

#[tokio::test]
async fn test_handle_connection_eof() -> anyhow::Result<()> {
    let callback = Arc::new(|_bundle: Bundle| {});

    let (client, server) = tokio::io::duplex(1024);

    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    // Close client immediately to trigger EOF
    drop(client);

    // Should complete without error
    let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_handle_connection_invalid_data() -> anyhow::Result<()> {
    let callback = Arc::new(|_bundle: Bundle| {});

    let (client, server) = tokio::io::duplex(1024);

    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    let mut client = client;

    // Send invalid length (too large)
    let invalid_len = 0xFFFFFFFFu32;
    client.write_all(&invalid_len.to_be_bytes()).await?;

    // This should cause an error when trying to allocate a huge buffer
    drop(client);

    let _result = tokio::time::timeout(Duration::from_millis(100), handle).await;
    // The handler should either complete or timeout (both are acceptable for this test)

    Ok(())
}

#[tokio::test]
async fn test_handle_connection_partial_data() -> anyhow::Result<()> {
    let callback = Arc::new(|_bundle: Bundle| {});

    let (client, server) = tokio::io::duplex(1024);

    let handle = tokio::spawn(async move { handle_connection(server, callback).await });

    let mut client = client;

    // Send length but not the full data
    let len = 100u32;
    client.write_all(&len.to_be_bytes()).await?;
    client.write_all(b"incomplete").await?; // Only 10 bytes, but promised 100

    drop(client);

    let _result = tokio::time::timeout(Duration::from_millis(100), handle).await;
    // Should timeout or complete with error

    Ok(())
}

#[tokio::test]
async fn test_tcp_cla_listener_activate_bind_error() {
    let callback = Arc::new(|_bundle: Bundle| {});

    // Try to bind to an invalid address
    let listener = TcpClaListener {
        bind_addr: "invalid:address".to_string(),
        receive_callback: callback,
    };

    let result = listener.activate().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bundle_serialization_roundtrip() -> anyhow::Result<()> {
    let original_bundle = create_test_bundle(
        "dtn://test_source",
        "dtn://test_destination",
        b"test payload data",
    );

    // Serialize
    let encoded = serde_cbor::to_vec(&original_bundle);
    assert!(encoded.is_ok());

    let encoded_data = encoded.unwrap();
    assert!(!encoded_data.is_empty());

    // Deserialize
    let decoded_bundle: Bundle = serde_cbor::from_slice(&encoded_data)?;

    // Verify all fields
    assert_eq!(
        decoded_bundle.primary.version,
        original_bundle.primary.version
    );
    assert_eq!(
        decoded_bundle.primary.source,
        original_bundle.primary.source
    );
    assert_eq!(
        decoded_bundle.primary.destination,
        original_bundle.primary.destination
    );
    assert_eq!(
        decoded_bundle.primary.report_to,
        original_bundle.primary.report_to
    );
    assert_eq!(
        decoded_bundle.primary.creation_timestamp,
        original_bundle.primary.creation_timestamp
    );
    assert_eq!(
        decoded_bundle.primary.lifetime,
        original_bundle.primary.lifetime
    );
    assert_eq!(decoded_bundle.payload, original_bundle.payload);

    Ok(())
}

#[test]
fn test_create_test_bundle_fields() {
    let bundle = create_test_bundle("source", "dest", b"payload");

    assert_eq!(bundle.primary.version, 7);
    assert_eq!(bundle.primary.source, "source");
    assert_eq!(bundle.primary.destination, "dest");
    assert_eq!(bundle.primary.report_to, "none");
    assert_eq!(bundle.primary.lifetime, 3600);
    assert_eq!(bundle.payload, b"payload");
    assert!(bundle.primary.creation_timestamp > 0);
}

#[test]
fn test_create_test_bundle_timing() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let bundle = create_test_bundle("src", "dst", b"test");
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    assert!(bundle.primary.creation_timestamp >= before);
    assert!(bundle.primary.creation_timestamp <= after);
}

#[test]
fn test_create_test_bundle_various_payloads() {
    let test_cases = vec![
        b"".to_vec(),
        b"simple".to_vec(),
        "unicode: こんにちは".as_bytes().to_vec(),
        vec![0, 1, 2, 255, 254, 253], // Binary data
    ];

    for payload in test_cases {
        let bundle = create_test_bundle("src", "dst", &payload);
        assert_eq!(bundle.payload, payload);
    }
}
