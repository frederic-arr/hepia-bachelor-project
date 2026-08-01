#![expect(clippy::unwrap_used, reason = "TODO")]
#![expect(clippy::print_stdout, reason = "TODO")]

use std::time::Duration;

use e2e::{CosVm, Vm, random_port, wait_for_request};

#[derive(Debug, Clone, Copy)]
#[expect(clippy::struct_field_names, reason = "")]
struct Measurement {
    time_to_config: Duration,
    time_to_install: Duration,
    time_to_kernel: Duration,
    time_to_init: Duration,
    time_to_supervisor: Duration,
    time_to_reconcile: Duration,
    time_to_dhcp: Duration,
    time_to_downloading_image: Duration,
    time_to_download_image: Duration,
    time_to_run_container: Duration,

    time_to_kernel_post: Duration,
    time_to_init_post: Duration,
    time_to_supervisor_post: Duration,
    time_to_reconcile_post: Duration,
    time_to_dhcp_post: Duration,
    time_to_run_container_post: Duration,
}

async fn cos_install() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/cos-install.yaml")
        .replace("%%PORT%%", &port.to_string());

    let mut vm = CosVm::new(
        Some(env!("CARGO_TARGET_TMPDIR")),
        Some(1024),
        vec![],
    )
    .await
    .unwrap();

    let console_socket = vm.vm.console_socket.clone();
    let start = vm.vm.start;
    let time_to_config = start.elapsed();
    let (a, time_to_install) = tokio::join!(vm.push_str(&data), async {
        Vm::wait_for_str(
            &console_socket,
            "install succesfull",
            Duration::from_secs(60),
        )
        .await
        .unwrap();
        start.elapsed()
    });
    a.unwrap();

    vm.wait_for_str("Linux version").await.unwrap();
    let time_to_kernel = vm.elapsed();

    vm.wait_for_str("Run /init as init process").await.unwrap();
    let time_to_init = vm.elapsed();

    vm.wait_for_str("/bin/supervisor").await.unwrap();
    let time_to_supervisor = vm.elapsed();

    vm.wait_for_str("attempting to reconcile").await.unwrap();
    let time_to_reconcile = vm.elapsed();

    vm.wait_for_str(
        "reconciled resource status=Ready key=network:route/eth0-dhcp",
    )
    .await
    .unwrap();
    let time_to_dhcp = vm.elapsed();

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

    tokio::time::sleep(Duration::from_secs(5)).await;
    let (
        time_to_kernel_post,
        time_to_init_post,
        time_to_supervisor_post,
        time_to_reconcile_post,
        time_to_dhcp_post,
    ) = vm.reboot().await.unwrap();

    wait_for_request(port).await.unwrap();
    let time_to_run_container_post = vm.elapsed();

    vm.kill().await.unwrap();

    Measurement {
        time_to_config,
        time_to_install,
        time_to_kernel,
        time_to_init,
        time_to_supervisor,
        time_to_reconcile,
        time_to_dhcp,
        time_to_downloading_image,
        time_to_download_image,
        time_to_run_container,

        time_to_kernel_post,
        time_to_init_post,
        time_to_supervisor_post,
        time_to_reconcile_post,
        time_to_dhcp_post,
        time_to_run_container_post,
    }
}

#[tokio::main(flavor = "local")]
async fn main() {
    const NUM_ITER: usize = 50;

    let mut cos = Vec::with_capacity(NUM_ITER);
    for i in 0..NUM_ITER {
        let data = cos_install().await;
        println!(
            "#{i}: {}s",
            data.time_to_run_container_post.as_secs()
        );
        cos.push(data);
    }

    std::fs::write(
        format!("{}/time-install.csv", env!("CARGO_TARGET_TMPDIR")),
        cos.into_iter()
            .map(|m| {
                format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    m.time_to_config.as_millis(),
                    m.time_to_install.as_millis(),
                    m.time_to_kernel.as_millis(),
                    m.time_to_init.as_millis(),
                    m.time_to_supervisor.as_millis(),
                    m.time_to_reconcile.as_millis(),
                    m.time_to_dhcp.as_millis(),
                    m.time_to_downloading_image.as_millis(),
                    m.time_to_download_image.as_millis(),
                    m.time_to_run_container.as_millis(),
                    m.time_to_kernel_post.as_millis(),
                    m.time_to_init_post.as_millis(),
                    m.time_to_supervisor_post.as_millis(),
                    m.time_to_reconcile_post.as_millis(),
                    m.time_to_dhcp_post.as_millis(),
                    m.time_to_run_container_post.as_millis(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
}
