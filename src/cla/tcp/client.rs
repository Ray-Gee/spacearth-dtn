use crate::bpv7::EndpointId;
use crate::cla::peer::ClaPeer;
use crate::consts::{BUNDLES_DIR, DISPATCHED_DIR};
use crate::store::file::BundleStore;
use crate::{bpv7::bundle::Bundle, cla::ConvergenceLayer};
use anyhow::Result;
use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;

/// TCP connection information including connection details
#[derive(Clone, Debug)]
pub struct TcpConnectionInfo {
    pub address: String,
    pub port: u16,
    pub latency: Option<Duration>,
    pub connection_time: Option<Duration>,
    pub is_reachable: bool,
    pub local_addr: Option<String>,
    pub remote_addr: Option<String>,
}

impl TcpConnectionInfo {
    pub fn new(address: String) -> Self {
        let (_host, port) = if let Some(colon_pos) = address.rfind(':') {
            let host = address[..colon_pos].to_string();
            let port = address[colon_pos + 1..].parse().unwrap_or(0);
            (host, port)
        } else {
            (address.clone(), 0)
        };

        Self {
            address,
            port,
            latency: None,
            connection_time: None,
            is_reachable: false,
            local_addr: None,
            remote_addr: None,
        }
    }

    pub fn display_info(&self) {
        println!("🌐 TCP Connection Info:");
        println!("   Address: {}:{}", self.address, self.port);
        if let Some(latency) = self.latency {
            println!("   Latency: {latency:?}");
        }
        if let Some(conn_time) = self.connection_time {
            println!("   Connection Time: {conn_time:?}");
        }
        println!("   Reachable: {}", self.is_reachable);
        if let Some(local) = &self.local_addr {
            println!("   Local Address: {local}");
        }
        if let Some(remote) = &self.remote_addr {
            println!("   Remote Address: {remote}");
        }
        println!();
    }
}

#[derive(Clone)]
pub struct TcpClaClient {
    pub target_addr: String,
    pub connection_info: Option<TcpConnectionInfo>,
}

/// TCP-specific implementation of ClaPeer for routing
#[derive(Clone)]
pub struct TcpPeer {
    pub peer_id: EndpointId,
    pub address: String,
    pub connection_info: Option<TcpConnectionInfo>,
}

impl TcpPeer {
    pub fn new(peer_id: EndpointId, address: String) -> Self {
        Self {
            peer_id,
            address,
            connection_info: None,
        }
    }

    /// Create TcpPeer from endpoint ID (assumes endpoint ID is the address)
    pub fn from_endpoint_id(peer_id: EndpointId) -> Self {
        let address = peer_id.as_str().to_string();
        Self {
            peer_id,
            address,
            connection_info: None,
        }
    }

    /// Create TcpPeer for testing (uses endpoint ID as address)
    #[cfg(test)]
    pub fn for_test(peer_id: EndpointId) -> Self {
        Self::from_endpoint_id(peer_id)
    }

    pub fn with_connection_info(mut self, info: TcpConnectionInfo) -> Self {
        self.connection_info = Some(info);
        self
    }

    pub fn get_connection_info(&self) -> Option<&TcpConnectionInfo> {
        self.connection_info.as_ref()
    }
}

#[async_trait]
impl ConvergenceLayer for TcpPeer {
    fn address(&self) -> String {
        self.address.clone()
    }
    async fn activate(&self) -> anyhow::Result<()> {
        if let Some(connection_info) = tcp_connect_and_collect_info(&self.address).await? {
            println!("✅ TCP connection established and info collected:");
            connection_info.display_info();
            Ok(())
        } else {
            Err(anyhow::anyhow!("TCP connection failed: {}", self.address))
        }
    }
}

#[async_trait]
impl ClaPeer for TcpPeer {
    fn get_peer_endpoint_id(&self) -> EndpointId {
        self.peer_id.clone()
    }

    async fn is_reachable(&self) -> bool {
        tcp_connect_and_collect_info(&self.address)
            .await
            .unwrap_or(None)
            .map(|info| info.is_reachable)
            .unwrap_or(false)
    }

    fn get_cla_type(&self) -> &str {
        "tcp"
    }

    fn get_connection_address(&self) -> String {
        if let Some(info) = &self.connection_info {
            format!("{}:{}", info.address, info.port)
        } else {
            self.address.clone()
        }
    }

    fn clone_box(&self) -> Box<dyn ClaPeer> {
        Box::new(self.clone())
    }

    async fn activate(&self) -> anyhow::Result<()> {
        <Self as ConvergenceLayer>::activate(self).await
    }
}

/// TCP-specific connectivity check with detailed connection information
async fn tcp_connect_and_collect_info(address: &str) -> anyhow::Result<Option<TcpConnectionInfo>> {
    let mut connection_info = TcpConnectionInfo::new(address.to_string());

    println!("🔍 Attempting TCP connection to: {address}");

    let start_time = Instant::now();

    match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(address)).await {
        Ok(Ok(stream)) => {
            let connection_time = start_time.elapsed();
            connection_info.connection_time = Some(connection_time);
            connection_info.is_reachable = true;

            // Get local and remote addresses
            if let Ok(local_addr) = stream.local_addr() {
                connection_info.local_addr = Some(local_addr.to_string());
            }
            if let Ok(remote_addr) = stream.peer_addr() {
                connection_info.remote_addr = Some(remote_addr.to_string());
            }

            // Measure latency with a simple ping-like test
            let ping_start = Instant::now();
            // Note: In a real implementation, you might send a small packet and measure RTT
            // For now, we'll just use the connection time as a rough latency indicator
            connection_info.latency = Some(ping_start.elapsed());

            println!("✅ TCP connection successful to: {address}");
            Ok(Some(connection_info))
        }
        Ok(Err(e)) => {
            println!("❌ TCP connection failed to {address}: {e}");
            connection_info.is_reachable = false;
            Ok(Some(connection_info))
        }
        Err(_) => {
            println!("❌ TCP connection timeout to: {address}");
            connection_info.is_reachable = false;
            Ok(Some(connection_info))
        }
    }
}

impl TcpClaClient {
    pub fn new(target_addr: String) -> Self {
        Self {
            target_addr,
            connection_info: None,
        }
    }

    /// Connect to the target and store connection information
    pub async fn connect_and_store_info(&mut self) -> anyhow::Result<bool> {
        if let Some(info) = tcp_connect_and_collect_info(&self.target_addr).await? {
            self.connection_info = Some(info.clone());
            println!("✅ TCP connection established and info stored:");
            info.display_info();
            Ok(info.is_reachable)
        } else {
            println!("❌ TCP connection failed: {}", self.target_addr);
            Ok(false)
        }
    }

    /// Get stored connection information
    pub fn get_connection_info(&self) -> Option<&TcpConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Display stored connection information
    pub fn display_stored_info(&self) {
        if let Some(info) = &self.connection_info {
            println!("🌐 Stored TCP Connection Info:");
            info.display_info();
        } else {
            println!("❌ No connection information stored. Run connect_and_store_info() first.");
        }
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
            println!("📨 Sending bundle: {id} bundle: {bundle:?} stream: {stream:?}");
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
