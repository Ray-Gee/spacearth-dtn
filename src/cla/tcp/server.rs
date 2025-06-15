use crate::bundle::Bundle;
use crate::cla::ConvergenceLayer;
use anyhow::Result;
use serde_cbor;
use std::sync::Arc;
use tokio::net::TcpListener;

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

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    callback: Arc<dyn Fn(Bundle) + Send + Sync>,
) -> Result<()> {
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
