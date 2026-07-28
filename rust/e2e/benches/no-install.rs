#![expect(clippy::unwrap_used, reason = "TODO")]

use std::time::{Duration, Instant};

use e2e::{CosVm, random_port, wait_for_request};

#[derive(Debug, Clone, Copy)]
struct Measurement {
    time_to_installer: Duration,
    time_to_run_container: Duration,
}

async fn create_vm() -> CosVm {
    CosVm::new(Some(env!("CARGO_TARGET_TMPDIR"))).await.unwrap()
}

async fn cos_no_install() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/cos-no-install.yaml")
        .replace("%%PORT%%", &port.to_string());

    let start = Instant::now();
    let mut vm = create_vm().await;
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
    const NUM_ITER: usize = 100;

    let mut cos_no_install_data = Vec::with_capacity(NUM_ITER);
    for _ in 0..NUM_ITER {
        let data = cos_no_install().await;
        dbg!(data);
        cos_no_install_data.push(data);
    }

    dbg!(cos_no_install_data);
}
