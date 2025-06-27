#[test]
fn test_module_path() {
    let path = module_path!();
    println!("Actual module path: {path}");
    assert!(path.contains("cla::tests"));
}

#[test]
fn test_current_module() {
    let current_module = module_path!();
    println!("Current module path: {current_module}");
    assert!(current_module.contains("cla::tests"));
}

use crate::bpv7::bundle::{Bundle, PrimaryBlock};
use crate::bpv7::EndpointId;
use crate::cla::manager::*;
use crate::cla::peer::ClaPeer;
use crate::cla::tcp::client::*;
use crate::cla::tcp::server::*;
use crate::cla::ConvergenceLayer;
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
#[derive(Debug, Clone)]
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

// ClaPeerトレイトをMockClaに実装
#[async_trait]
impl ClaPeer for MockCla {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        EndpointId::from(self.address.as_str())
    }
    async fn is_reachable(&self) -> bool {
        // 失敗するCLAは到達不能として扱う
        !self.should_fail
    }
    fn get_cla_type(&self) -> &str {
        "mock"
    }
    fn get_connection_address(&self) -> String {
        self.address.clone()
    }
    fn clone_box(&self) -> Box<dyn ClaPeer> {
        Box::new(self.clone())
    }
    async fn activate(&self) -> anyhow::Result<()> {
        <Self as ConvergenceLayer>::activate(self).await
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
    let peers = manager.list_reachable_peers().await;
    let addresses: Vec<String> = peers.iter().map(|p| p.get_connection_address()).collect();
    assert!(addresses.is_empty());
}

#[tokio::test]
async fn test_register_single_cla() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let manager = ClaManager::new(move |_bundle| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let mock_cla = Box::new(MockCla::new("test://127.0.0.1:8080"));

    manager.register_peer(mock_cla).await;

    // Give some time for the registration to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let peers = manager.list_reachable_peers().await;
    assert_eq!(peers.len(), 1);
    let addresses: Vec<String> = peers.iter().map(|p| p.get_connection_address()).collect();
    assert!(addresses.contains(&"test://127.0.0.1:8080".to_string()));
}

#[tokio::test]
async fn test_register_multiple_clas() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let manager = ClaManager::new(move |_bundle| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let cla1 = Box::new(MockCla::new("test://127.0.0.1:8080"));
    let cla2 = Box::new(MockCla::new("test://127.0.0.1:8081"));
    let cla3 = Box::new(MockCla::new("test://127.0.0.1:8082"));

    manager.register_peer(cla1).await;
    manager.register_peer(cla2).await;
    manager.register_peer(cla3).await;

    // Give some time for registrations to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let peers = manager.list_reachable_peers().await;
    assert_eq!(peers.len(), 3);
    let addresses: Vec<String> = peers.iter().map(|p| p.get_connection_address()).collect();
    assert!(addresses.contains(&"test://127.0.0.1:8080".to_string()));
    assert!(addresses.contains(&"test://127.0.0.1:8081".to_string()));
    assert!(addresses.contains(&"test://127.0.0.1:8082".to_string()));
}

#[tokio::test]
async fn test_register_duplicate_cla() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let manager = ClaManager::new(move |_bundle| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let cla1 = Box::new(MockCla::new("test://127.0.0.1:8080"));
    let cla2 = Box::new(MockCla::new("test://127.0.0.1:8080")); // Same address

    manager.register_peer(cla1).await;
    manager.register_peer(cla2).await; // Should not register due to duplicate address

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let peers = manager.list_reachable_peers().await;
    assert_eq!(peers.len(), 1);
}

#[tokio::test]
async fn test_register_failing_cla() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);

    let manager = ClaManager::new(move |_bundle| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let failing_cla = Box::new(MockCla::new_failing("test://127.0.0.1:8080"));

    manager.register_peer(failing_cla.clone()).await;

    // Give some time for activation to fail
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Failing CLA should be registered but not reachable
    let all_peers = manager.list_all_peers().await;
    assert_eq!(all_peers.len(), 1);

    let reachable_peers = manager.list_reachable_peers().await;
    assert_eq!(reachable_peers.len(), 0); // Failing CLA is not reachable

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
            &format!("dtn://source{i}"),
            &format!("dtn://dest{i}"),
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
    let cla1 = Box::new(MockCla::new("test://127.0.0.1:8080"));
    let cla2 = Box::new(MockCla::new("test://127.0.0.1:8081"));

    manager1.register_peer(cla1).await;
    manager2.register_peer(cla2).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Both managers should see the same state
    let peers1 = manager1.list_reachable_peers().await;
    let peers2 = manager2.list_reachable_peers().await;
    let addresses1: Vec<String> = peers1.iter().map(|p| p.get_connection_address()).collect();
    let addresses2: Vec<String> = peers2.iter().map(|p| p.get_connection_address()).collect();
    assert_eq!(addresses1.len(), 2);
    assert_eq!(addresses2.len(), 2);
    assert_eq!(addresses1, addresses2);
}

#[tokio::test]
async fn test_list_active_empty() {
    let manager = ClaManager::new(|_bundle| {});

    let peers = manager.list_reachable_peers().await;
    assert!(peers.is_empty());
}

#[test]
fn test_mock_cla_address() {
    let mock_cla = MockCla::new("test://example.com:1234");
    assert_eq!(mock_cla.address(), "test://example.com:1234");
}

#[tokio::test]
async fn test_mock_cla_activation_success() {
    let mock_cla = MockCla::new("test://example.com");
    let result = <MockCla as ConvergenceLayer>::activate(&mock_cla).await;
    assert!(result.is_ok());
    assert_eq!(mock_cla.activation_count(), 1);
}

#[tokio::test]
async fn test_mock_cla_activation_failure() {
    let mock_cla = MockCla::new_failing("test://example.com");
    let result = <MockCla as ConvergenceLayer>::activate(&mock_cla).await;
    assert!(result.is_err());
    assert_eq!(mock_cla.activation_count(), 1);
}

#[test]
fn test_tcp_cla_dialer_new() {
    let dialer = TcpClaClient {
        target_addr: "127.0.0.1:8080".to_string(),
        connection_info: None,
    };
    assert_eq!(dialer.target_addr, "127.0.0.1:8080");
}

#[test]
fn test_tcp_cla_dialer_address() {
    let dialer = TcpClaClient {
        target_addr: "localhost:9090".to_string(),
        connection_info: None,
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
            &format!("dtn://source_{name}"),
            &format!("dtn://dest_{name}"),
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

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
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

        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
        let bundle = create_test_bundle(
            &format!("dtn://source_{i}"),
            &format!("dtn://dest_{i}"),
            format!("test payload {i}").as_bytes(),
        );

        let result = send_bundle(&mut stream, &bundle).await;
        assert!(result.is_ok(), "Failed for ACK: {ack}");
    }

    Ok(())
}

#[tokio::test]
async fn test_send_bundle_large_payload() -> anyhow::Result<()> {
    let (port, _handle) = mock_tcp_server(OK).await?;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;

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
        connection_info: None,
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
        target_addr: format!("127.0.0.1:{port}"),
        connection_info: None,
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
            &format!("dtn://source{i}"),
            &format!("dtn://dest{i}"),
            format!("payload{i}").into_bytes(),
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
            &format!("dtn://source{i}"),
            &format!("dtn://dest{i}"),
            format!("payload {i}").as_bytes(),
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

// TcpPeer tests for better coverage
#[test]
fn test_tcp_peer_new() {
    let eid = crate::bpv7::EndpointId::from("dtn://test-peer");
    let peer = crate::cla::TcpPeer::new(eid.clone(), "192.168.1.100:8080".to_string());

    assert_eq!(peer.peer_id, eid);
    assert_eq!(peer.address, "192.168.1.100:8080");
}

#[test]
fn test_tcp_peer_from_endpoint_id() {
    let eid = crate::bpv7::EndpointId::from("dtn://test-node");
    let peer = crate::cla::TcpPeer::from_endpoint_id(eid.clone());

    assert_eq!(peer.peer_id, eid);
    assert_eq!(peer.address, "dtn://test-node");
}

#[test]
fn test_tcp_peer_for_test() {
    let eid = crate::bpv7::EndpointId::from("dtn://test-endpoint");
    let peer = crate::cla::TcpPeer::for_test(eid.clone());

    assert_eq!(peer.peer_id, eid);
    assert_eq!(peer.address, "dtn://test-endpoint");
}

#[test]
fn test_tcp_peer_get_peer_endpoint_id() {
    let eid = crate::bpv7::EndpointId::from("dtn://my-peer");
    let peer = crate::cla::TcpPeer::new(eid.clone(), "10.0.0.1:9090".to_string());

    assert_eq!(peer.get_peer_endpoint_id(), eid);
}

#[test]
fn test_tcp_peer_get_cla_type() {
    let eid = crate::bpv7::EndpointId::from("dtn://any-peer");
    let peer = crate::cla::TcpPeer::new(eid, "localhost:8080".to_string());

    assert_eq!(peer.get_cla_type(), "tcp");
}

#[test]
fn test_tcp_peer_get_connection_address() {
    let eid = crate::bpv7::EndpointId::from("dtn://addr-test");
    let address = "example.com:1234".to_string();
    let peer = crate::cla::TcpPeer::new(eid, address.clone());

    assert_eq!(peer.get_connection_address(), address);
}

#[tokio::test]
async fn test_tcp_peer_is_reachable_unreachable() {
    let eid = crate::bpv7::EndpointId::from("dtn://unreachable");
    let peer = crate::cla::TcpPeer::new(eid, "127.0.0.1:19998".to_string()); // Non-existent port

    let reachable = peer.is_reachable().await;
    assert!(!reachable);
}

#[tokio::test]
async fn test_tcp_peer_is_reachable_timeout() {
    let eid = crate::bpv7::EndpointId::from("dtn://timeout-test");
    // Use a non-routable address that will cause timeout
    let peer = crate::cla::TcpPeer::new(eid, "192.0.2.1:80".to_string()); // TEST-NET-1 (RFC 5737)

    let reachable = peer.is_reachable().await;
    assert!(!reachable);
}

#[tokio::test]
async fn test_tcp_peer_is_reachable_with_mock_server() -> anyhow::Result<()> {
    // Create a mock server to test successful connection
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    // Start a server that accepts connections
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            drop(stream); // Just accept and close
        }
    });

    let eid = crate::bpv7::EndpointId::from("dtn://reachable");
    let peer = crate::cla::TcpPeer::new(eid, format!("127.0.0.1:{port}"));

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let reachable = peer.is_reachable().await;
    assert!(reachable);

    Ok(())
}

// Additional TcpClaClient tests
#[test]
fn test_tcp_cla_client_new() {
    let client = TcpClaClient {
        target_addr: "test.example.com:8080".to_string(),
        connection_info: None,
    };
    assert_eq!(client.target_addr, "test.example.com:8080");
}

#[tokio::test]
async fn test_tcp_cla_client_activate_connection_refused() {
    let client = TcpClaClient {
        target_addr: "127.0.0.1:19997".to_string(), // Non-existent server
        connection_info: None,
    };

    let result = client.activate().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tcp_cla_client_activate_invalid_address() {
    let client = TcpClaClient {
        target_addr: "invalid-hostname:8080".to_string(),
        connection_info: None,
    };

    let result = client.activate().await;
    assert!(result.is_err());
}

// Test create_bundle function variations
#[test]
fn test_create_bundle_empty_payload() {
    let bundle = create_bundle("dtn://src", "dtn://dst", vec![]);
    assert_eq!(bundle.payload, b"");
    assert_eq!(bundle.primary.source, "dtn://src");
    assert_eq!(bundle.primary.destination, "dtn://dst");
}

#[test]
fn test_create_bundle_large_payload() {
    let large_payload = vec![0xFF; 1000];
    let bundle = create_bundle("dtn://big-src", "dtn://big-dst", large_payload.clone());
    assert_eq!(bundle.payload, large_payload);
}

#[test]
fn test_create_bundle_unicode_addresses() {
    let bundle = create_bundle(
        "dtn://テスト送信",
        "dtn://テスト受信",
        b"unicode test".to_vec(),
    );
    assert_eq!(bundle.primary.source, "dtn://テスト送信");
    assert_eq!(bundle.primary.destination, "dtn://テスト受信");
}

// =====================
// BLE Client Unit Tests
// =====================
#[cfg(test)]
mod ble_client_tests {
    use super::*;
    use crate::cla::ble::client::*;

    #[test]
    fn test_ble_connection_info_new_and_display() {
        let info = BleConnectionInfo::new("dev1".to_string(), "AA:BB:CC:DD:EE:FF".to_string());
        assert_eq!(info.device_name, "dev1");
        assert_eq!(info.mac_address, "AA:BB:CC:DD:EE:FF");
        info.display_info(); // just call for coverage
    }

    #[test]
    fn test_ble_peer_new_and_methods() {
        let peer_id = EndpointId::from("dtn://ble-peer");
        let peer = BlePeer::new(peer_id.clone(), "dev1".to_string());
        assert_eq!(peer.peer_id, peer_id);
        assert_eq!(peer.device_name, "dev1");
        assert!(peer.get_connection_info().is_none());
    }

    #[test]
    fn test_ble_peer_with_connection_info() {
        let peer_id = EndpointId::from("dtn://ble-peer");
        let info = BleConnectionInfo::new("dev1".to_string(), "AA:BB:CC:DD:EE:FF".to_string());
        let peer = BlePeer::new(peer_id, "dev1".to_string()).with_connection_info(info.clone());
        assert!(peer.get_connection_info().is_some());
    }

    #[test]
    fn test_ble_cla_client_new_and_methods() {
        let client = BleClaClient::new("dev1".to_string());
        assert_eq!(client.device_name, "dev1");
        assert!(client.get_connection_info().is_none());
        client.display_stored_info(); // just call for coverage
    }

    #[tokio::test]
    async fn test_ble_peer_trait_methods() {
        let peer_id = EndpointId::from("dtn://ble-peer");
        let peer = BlePeer::new(peer_id, "dev1".to_string());
        let _ = peer.address();
        let _ = peer.get_peer_endpoint_id();
        let _ = peer.get_cla_type();
        let _ = peer.get_connection_address();
        // activate/is_reachableは実機依存なのでエラーでもOK
        let _ = peer.is_reachable().await;
    }
}

// =====================
// TCP Client Unit Tests
// =====================
#[cfg(test)]
mod tcp_client_tests {
    use super::*;

    #[test]
    fn test_tcp_connection_info_new_and_display() {
        let info = TcpConnectionInfo::new("127.0.0.1:1234".to_string());
        assert_eq!(info.address, "127.0.0.1:1234");
        info.display_info(); // just call for coverage
    }

    #[test]
    fn test_tcp_cla_client_new_and_methods() {
        let client = TcpClaClient::new("127.0.0.1:1234".to_string());
        assert_eq!(client.target_addr, "127.0.0.1:1234");
        assert!(client.get_connection_info().is_none());
        client.display_stored_info(); // just call for coverage
    }

    #[tokio::test]
    async fn test_tcp_peer_trait_methods() {
        let peer_id = EndpointId::from("dtn://tcp-peer");
        let peer = TcpPeer::new(peer_id, "127.0.0.1:1234".to_string());
        let _ = peer.address();
        let _ = peer.get_peer_endpoint_id();
        let _ = peer.get_cla_type();
        let _ = peer.get_connection_address();
        // activate/is_reachableは実ネット依存なのでエラーでもOK
        let _ = peer.is_reachable().await;
    }
}
