#![expect(clippy::unwrap_used, reason = "TODO")]

use std::time::{Duration, Instant};

use e2e::{CosVm, random_port, wait_for_request};

#[derive(Debug, Clone, Copy)]
struct Measurement {
    time_to_installer: Duration,
    time_to_run_container: Duration,
}

async fn cos_install() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/cos-install.yaml")
        .replace("%%PORT%%", &port.to_string());

    let start = Instant::now();
    let mut vm = CosVm::new(Some(env!("CARGO_TARGET_TMPDIR")), Some(512))
        .await
        .unwrap();
    let time_to_installer = start.elapsed();

    vm.push_str(&data).await.unwrap();

    wait_for_request(port).await.unwrap();
    let time_to_run_container = start.elapsed();

    vm.kill().await.unwrap();

    Measurement {
        time_to_installer,
        time_to_run_container,
    }
}

#[tokio::main(flavor = "local")]
async fn main() {
    const NUM_ITER: usize = 1;

    let mut cos = Vec::with_capacity(NUM_ITER);
    for _ in 0..NUM_ITER {
        let data = cos_install().await;
        cos.push(data);
    }

    std::fs::write(
        format!("{}/time-install.csv", env!("CARGO_TARGET_TMPDIR")),
        cos.into_iter()
            .map(|m| {
                format!(
                    "{},{}",
                    m.time_to_installer.as_millis(),
                    m.time_to_run_container.as_millis()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
