#![feature(exit_status_error)]

mod cos;
mod vm;
use anyhow::Result;
pub use cos::*;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
pub use vm::*;

#[must_use]
pub fn random_port() -> u16 {
    rand::random_range(32768..60999)
}

pub async fn wait_for_request(port: u16) -> Result<String> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await?;

    let (mut socket, _) = listener.accept().await?;
    let mut buf = [0_u8; 1024];
    let _ = socket.read(&mut buf).await;

    let response = "HTTP/1.1 200 OK\r\nContent-Type: \
                    text/plain\r\nContent-Length: 16\r\nConnection: \
                    close\r\n\r\nhello from VM\n";

    socket.write_all(response.as_bytes()).await?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}
