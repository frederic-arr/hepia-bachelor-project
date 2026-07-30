#![expect(clippy::unwrap_used, reason = "TODO")]
#![expect(clippy::print_stderr, reason = "TODO")]

use anyhow::Result;
use e2e::{CosVm, random_port};
use tokio::io::AsyncReadExt as _;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Copy)]
struct Measurement {
    alloc: usize,
}

pub async fn membench(port: u16) -> Result<usize> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await?;
    let (mut socket, _) = listener.accept().await?;
    let mut buf = [0_u8; 1024];

    let mut max = 0;
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        let Some(line) = std::str::from_utf8(&buf)?.lines().next() else {
            break;
        };

        if line == "STOP" {
            break;
        }

        let size: usize = line.parse()?;
        if size > max {
            max = size;
        } else {
            eprintln!("size smaller that previous: max={max}   size={size}");
            break;
        }
    }

    Ok(max)
}

async fn cos_membench() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/cos-membench.yaml")
        .replace("%%PORT%%", &port.to_string());

    let mut vm = CosVm::new(Some(env!("CARGO_TARGET_TMPDIR")), None)
        .await
        .unwrap();

    vm.push_str(&data).await.unwrap();

    let max = membench(port).await.unwrap();

    vm.kill().await.unwrap();

    Measurement { alloc: max }
}

#[tokio::main(flavor = "local")]
async fn main() {
    const NUM_ITER: usize = 100;

    let mut cos = Vec::with_capacity(NUM_ITER);
    for _ in 0..NUM_ITER {
        let data = cos_membench().await;
        cos.push(data);
    }

    std::fs::write(
        format!("{}/membench.csv", env!("CARGO_TARGET_TMPDIR")),
        cos.into_iter()
            .map(|m| format!("{}", m.alloc))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
