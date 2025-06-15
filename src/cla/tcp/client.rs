use crate::store::file::BundleStore;
use crate::{bundle::Bundle, cla::ConvergenceLayer};
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

        let store = BundleStore::new("./bundles")?;
        let dispatched_dir = std::path::Path::new("./dispatched");

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
