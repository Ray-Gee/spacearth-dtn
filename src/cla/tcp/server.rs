use crate::cla::ConvergenceLayer;
use anyhow::Result;
use tokio::net::TcpListener;

pub struct TcpClaListener {
    pub bind_addr: String,
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

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream).await {
                    eprintln!("Connection error: {:?}", e);
                }
            });
        }
    }
}

async fn handle_connection(mut stream: tokio::net::TcpStream) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    println!("Received: {:?}", &buf[..n]);

    stream.write_all(b"ACK").await?;
    Ok(())
}
