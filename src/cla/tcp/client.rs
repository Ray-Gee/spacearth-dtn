use crate::bpv7::EndpointId;
use crate::consts::{BUNDLES_DIR, DISPATCHED_DIR};
use crate::routing::algorithm::ClaPeer;
use crate::store::file::BundleStore;
use crate::{bpv7::bundle::Bundle, cla::ConvergenceLayer};
use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;

pub struct TcpClaClient {
    pub target_addr: String,
}

/// TCP-specific implementation of ClaPeer for routing
pub struct TcpPeer {
    pub peer_id: EndpointId,
    pub address: String,
}

impl TcpPeer {
    pub fn new(peer_id: EndpointId, address: String) -> Self {
        Self { peer_id, address }
    }

    /// Create TcpPeer from endpoint ID (assumes endpoint ID is the address)
    pub fn from_endpoint_id(peer_id: EndpointId) -> Self {
        let address = peer_id.as_str().to_string();
        Self { peer_id, address }
    }

    /// Create TcpPeer for testing (uses endpoint ID as address)
    #[cfg(test)]
    pub fn for_test(peer_id: EndpointId) -> Self {
        Self::from_endpoint_id(peer_id)
    }
}

#[async_trait]
impl ClaPeer for TcpPeer {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        self.peer_id.clone()
    }

    async fn is_reachable(&self) -> bool {
        // Try to establish a TCP connection to check reachability
        // Use a short timeout to avoid blocking too long
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            TcpStream::connect(&self.address),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                println!(
                    "✅ TCP peer {} ({}) is reachable",
                    self.peer_id, self.address
                );
                true
            }
            Ok(Err(e)) => {
                println!(
                    "❌ TCP peer {} ({}) connection failed: {}",
                    self.peer_id, self.address, e
                );
                false
            }
            Err(_) => {
                println!(
                    "❌ TCP peer {} ({}) connection timed out",
                    self.peer_id, self.address
                );
                false
            }
        }
    }

    fn get_cla_type(&self) -> &str {
        "tcp"
    }

    fn get_connection_address(&self) -> String {
        self.address.clone()
    }
}

#[async_trait::async_trait]
impl ConvergenceLayer for TcpClaClient {
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
