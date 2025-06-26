use crate::bpv7::bundle::Bundle;
use crate::cla::ConvergenceLayer;
use anyhow::Result;
use serde_cbor;
use std::sync::Arc;
use tokio::net::TcpListener;

// TODO: receive_callbackがClaManagerとTcpClaListenerの両方で保持されている
// 設計を見直して、コールバックの責任を一箇所に集約する必要がある
// 例: ClaManagerが全てのCLAのコールバックを管理し、各CLAは単純にデータを転送するだけにする
#[derive(Clone)]
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
        println!("TCP CLA Listener listening on {}", self.bind_addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("📨 New connection from: {}", addr);

            let callback = Arc::clone(&self.receive_callback);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, callback).await {
                    eprintln!("❌ Error handling connection: {}", e);
                }
            });
        }
    }
}

pub async fn handle_connection<S>(
    mut stream: S,
    callback: Arc<dyn Fn(Bundle) + Send + Sync>,
) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    loop {
        // Read length
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(_) => break, // EOF or connection closed
        }

        let len = u32::from_be_bytes(len_buf) as usize;

        // Read bundle data
        let mut data = vec![0u8; len];
        if stream.read_exact(&mut data).await.is_err() {
            break;
        }

        // Deserialize bundle
        match serde_cbor::from_slice::<Bundle>(&data) {
            Ok(bundle) => {
                // Call the callback
                callback(bundle);

                // Send OK response
                let _ = stream.write_all(b"OK").await;
            }
            Err(e) => {
                eprintln!("❌ Failed to deserialize bundle: {}", e);
                let _ = stream.write_all(b"ERROR").await;
            }
        }
    }

    Ok(())
}
