use crate::cla::ConvergenceLayer;
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

        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        stream.write_all(b"Hello from Dialer").await?;

        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).await?;
        println!("Received ACK: {:?}", &buf[..n]);

        Ok(())
    }
}
