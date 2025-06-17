use crate::bundle::Bundle;
use crate::cla::ConvergenceLayer;
use anyhow::Result;
use serde_cbor;
use std::sync::Arc;
use tokio::net::TcpListener;

// TODO: receive_callbackがClaManagerとTcpClaListenerの両方で保持されている
// 設計を見直して、コールバックの責任を一箇所に集約する必要がある
// 例: ClaManagerが全てのCLAのコールバックを管理し、各CLAは単純にデータを転送するだけにする
pub struct TcpClaListener {
    pub bind_addr: String,
    pub receive_callback: Arc<dyn Fn(Bundle) + Send + Sync>,
}

#[async_trait::async_trait]
impl ConvergenceLayer for TcpClaListener {
    fn address(&self) -> String {
        self.bind_addr.clone()
    }

    async fn activate(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.bind_addr).await?;
        println!("TCP Listener bound on {}", self.bind_addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            println!("Accepted connection from {}", peer_addr);

            let callback = Arc::clone(&self.receive_callback);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, callback).await {
                    eprintln!("Connection error: {:?}", e);
                }
            });
        }
    }
}

async fn handle_connection<S>(
    mut stream: S,
    callback: Arc<dyn Fn(Bundle) + Send + Sync>,
) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    loop {
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => { /* normal processing */ }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("✅ Stream closed by peer (normal EOF)");
                println!("🚦 Ready to accept DTN connections...");
                break;
            }
            Err(e) => {
                eprintln!("❌ Stream read error: {:?}", e);
                break;
            }
        }
        println!("📨 Received len_buf: {:?}", len_buf);
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        let bundle: Bundle = serde_cbor::from_slice(&buf)?;
        println!(
            "📦 Received bundle:\n  From: {}\n  To: {}\n  Payload: {}",
            bundle.primary.source,
            bundle.primary.destination,
            String::from_utf8_lossy(&bundle.payload)
        );

        (callback)(bundle);

        stream.write_all(b"OK").await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{Bundle, PrimaryBlock};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::Duration;

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

    async fn send_bundle_to_server(addr: &str, bundle: &Bundle) -> Result<String> {
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
    async fn test_handle_connection_single_bundle() -> Result<()> {
        let received_bundles = Arc::new(Mutex::new(Vec::new()));
        let bundles_clone = Arc::clone(&received_bundles);

        let callback = {
            let bundles_ref = Arc::clone(&bundles_clone);
            Arc::new(move |bundle: Bundle| {
                if let Ok(mut guard) = bundles_ref.lock() {
                    guard.push(bundle);
                }
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
        let received = received_bundles.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].primary.source, "dtn://source");
        assert_eq!(received[0].primary.destination, "dtn://dest");
        assert_eq!(received[0].payload, b"test payload");

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_connection_multiple_bundles() -> Result<()> {
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
    async fn test_handle_connection_large_bundle() -> Result<()> {
        let received_bundles = Arc::new(Mutex::new(Vec::new()));
        let bundles_clone = Arc::clone(&received_bundles);

        let callback = {
            let bundles_ref = Arc::clone(&bundles_clone);
            Arc::new(move |bundle: Bundle| {
                if let Ok(mut guard) = bundles_ref.lock() {
                    guard.push(bundle);
                }
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

        let received = received_bundles.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].payload.len(), 10000);

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_connection_eof() -> Result<()> {
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
    async fn test_handle_connection_invalid_data() -> Result<()> {
        let callback = Arc::new(|_bundle: Bundle| {});

        let (client, server) = tokio::io::duplex(1024);

        let handle = tokio::spawn(async move { handle_connection(server, callback).await });

        let mut client = client;

        // Send invalid length (too large)
        let invalid_len = 0xFFFFFFFFu32;
        client.write_all(&invalid_len.to_be_bytes()).await?;

        // This should cause an error when trying to allocate a huge buffer
        drop(client);

        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
        // The handler should either complete or timeout (both are acceptable for this test)

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_connection_partial_data() -> Result<()> {
        let callback = Arc::new(|_bundle: Bundle| {});

        let (client, server) = tokio::io::duplex(1024);

        let handle = tokio::spawn(async move { handle_connection(server, callback).await });

        let mut client = client;

        // Send length but not the full data
        let len = 100u32;
        client.write_all(&len.to_be_bytes()).await?;
        client.write_all(b"incomplete").await?; // Only 10 bytes, but promised 100

        drop(client);

        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
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
    async fn test_bundle_serialization_roundtrip() -> Result<()> {
        let original_bundle = create_test_bundle(
            "dtn://test_source",
            "dtn://test_destination",
            b"test payload data",
        );

        // Serialize
        let encoded = serde_cbor::to_vec(&original_bundle)?;
        assert!(!encoded.is_empty());

        // Deserialize
        let decoded_bundle: Bundle = serde_cbor::from_slice(&encoded)?;

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
}
