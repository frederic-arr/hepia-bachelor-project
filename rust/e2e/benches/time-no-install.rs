#![expect(clippy::unwrap_used, reason = "TODO")]
#![expect(clippy::print_stdout, reason = "TODO")]

use std::time::Duration;

use e2e::{CosVm, random_port, wait_for_request};

#[derive(Debug, Clone, Copy)]
#[expect(clippy::struct_field_names, reason = "")]
struct Measurement {
    time_to_kernel: Duration,
    time_to_init: Duration,
    time_to_supervisor: Duration,
    time_to_reconcile: Duration,
    time_to_dhcp: Duration,
    time_to_downloading_image: Duration,
    time_to_download_image: Duration,
    time_to_run_container: Duration,
}

async fn cos_no_install() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/cos-no-install.yaml")
        .replace("%%PORT%%", &port.to_string());

    let mut vm = CosVm::new(Some(env!("CARGO_TARGET_TMPDIR")), None, vec![])
        .await
        .unwrap();

    vm.push_str(&data).await.unwrap();

    vm.wait_for_str("attempting to reconcile container:image/")
        .await
        .unwrap();
    let time_to_downloading_image = vm.elapsed();

    vm.wait_for_str("econciled resource status=Done key=container:image/")
        .await
        .unwrap();
    let time_to_download_image = vm.elapsed();

    wait_for_request(port).await.unwrap();
    let time_to_run_container = vm.elapsed();

    vm.kill().await.unwrap();

    Measurement {
        time_to_kernel: vm.time_to_kernel,
        time_to_init: vm.time_to_init,
        time_to_supervisor: vm.time_to_supervisor,
        time_to_reconcile: vm.time_to_reconcile,
        time_to_dhcp: vm.time_to_dhcp,
        time_to_downloading_image,
        time_to_download_image,
        time_to_run_container,
    }
}

#[tokio::main(flavor = "local")]
async fn main() {
    const NUM_ITER: usize = 100;

    let mut cos = Vec::with_capacity(NUM_ITER);
    for i in 0..NUM_ITER {
        let data = cos_no_install().await;
        println!("#{i}: {}s", data.time_to_run_container.as_secs());
        cos.push(data);
    }

    std::fs::write(
        format!(
            "{}/time-no-install.csv",
            env!("CARGO_TARGET_TMPDIR")
        ),
        cos.into_iter()
            .map(|m| {
                format!(
                    "{},{},{},{},{},{},{},{}",
                    m.time_to_kernel.as_millis(),
                    m.time_to_init.as_millis(),
                    m.time_to_supervisor.as_millis(),
                    m.time_to_reconcile.as_millis(),
                    m.time_to_dhcp.as_millis(),
                    m.time_to_downloading_image.as_millis(),
                    m.time_to_download_image.as_millis(),
                    m.time_to_run_container.as_millis(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
