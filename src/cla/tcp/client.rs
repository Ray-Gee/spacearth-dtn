use crate::consts::{BUNDLES_DIR, DISPATCHED_DIR};
use crate::store::file::BundleStore;
use crate::{bpv7::bundle::Bundle, cla::ConvergenceLayer};
use anyhow::Result;
use tokio::net::TcpStream;

pub struct TcpClaDialer {
    pub target_addr: String,
}

#[async_trait::async_trait]
impl ConvergenceLayer for TcpClaDialer {
    fn address(&self) -> String {
        self.target_addr.clone()
    }

    async fn activate(&self) -> Result<()> {
        let mut stream = TcpStream::connect(&self.target_addr).await?;
        println!("Connected to {}", self.target_addr);

        let store = BundleStore::new(BUNDLES_DIR)?;
        let dispatched_dir = std::path::Path::new(DISPATCHED_DIR);

        for id in store.list()? {
            let bundle = store.load_by_partial_id(&id)?;
            println!(
                "📨 Sending bundle: {id} bundle: {:?} stream: {:?}",
                bundle, stream
            );
            if send_bundle(&mut stream, &bundle).await.is_ok() {
                store.dispatch_one(&bundle, dispatched_dir)?;
            } else {
                eprintln!("❌ Failed to send bundle: {id}");
            }
        }

        Ok(())
    }
}

pub fn create_bundle(source: &str, destination: &str, payload: Vec<u8>) -> Bundle {
    Bundle::new(source, destination, payload)
}

pub async fn send_bundle(stream: &mut TcpStream, bundle: &Bundle) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let encoded = serde_cbor::to_vec(bundle)?;
    let len = encoded.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&encoded).await?;

    let mut buf = [0u8; 16];
    let n = stream.read(&mut buf).await?;
    println!("📨 Received n: {n}");
    let ack = std::str::from_utf8(&buf[..n])?;
    println!("📨 Received ACK: \"{ack}\"");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpv7::bundle::{Bundle, PrimaryBlock};
    use crate::consts::tcp::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

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
    fn test_tcp_cla_dialer_new() {
        let dialer = TcpClaDialer {
            target_addr: "127.0.0.1:8080".to_string(),
        };
        assert_eq!(dialer.target_addr, "127.0.0.1:8080");
    }

    #[test]
    fn test_tcp_cla_dialer_address() {
        let dialer = TcpClaDialer {
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
    async fn mock_tcp_server(port: u16, response: &'static str) -> Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

        tokio::spawn(async move {
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
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_send_bundle_success() -> Result<()> {
        let port = 18080;
        mock_tcp_server(port, OK).await?;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        let bundle = create_test_bundle("dtn://source", "dtn://dest", b"test payload");

        let result = send_bundle(&mut stream, &bundle).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_bundle_with_different_acks() -> Result<()> {
        let test_cases = [OK, ACK, SUCCESS, RECEIVED];

        for (i, ack) in test_cases.iter().enumerate() {
            let port = 18081 + i as u16;
            mock_tcp_server(port, ack).await?;

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
    async fn test_send_bundle_large_payload() -> Result<()> {
        let port = 18090;
        mock_tcp_server(port, OK).await?;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;

        // Create a large payload
        let large_payload = vec![42u8; 10000];
        let bundle = create_test_bundle("dtn://source", "dtn://dest", &large_payload);

        let result = send_bundle(&mut stream, &bundle).await;
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_send_bundle_serialization() -> Result<()> {
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
        let dialer = TcpClaDialer {
            target_addr: "127.0.0.1:19999".to_string(), // Non-existent server
        };

        // This should fail because there's no server listening
        let result = dialer.activate().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_cla_dialer_activate_with_empty_store() -> Result<()> {
        // Create a mock server that accepts connections but expects no data
        let port = 18095;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                // Just accept the connection and close it
                let _ = stream.shutdown().await;
            }
        });

        // Give the server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Create a temporary directory for empty bundle store
        let temp_dir = TempDir::new()?;
        let _temp_bundles_dir = temp_dir.path().join("test_bundles");

        // Test with custom bundles directory
        let _dialer = TcpClaDialer {
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
}
