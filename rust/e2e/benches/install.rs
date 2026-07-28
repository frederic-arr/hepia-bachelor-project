#![expect(clippy::unwrap_used, reason = "TODO")]

use std::time::{Duration, Instant};

use e2e::{CosVm, Vm, random_port, wait_for_request};
use tokio::process::Command;

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

async fn talos_install() -> Measurement {
    let port = random_port();
    let data = include_str!("./data/talos.yaml")
        .replace("%%PORT%%", &port.to_string());

    let start = Instant::now();
    let vm = Vm::new(
        Some(env!("CARGO_TARGET_TMPDIR")),
        "./benches/data/talos.iso",
        50000,
        // Some(1024),
        None,
        2048,
    )
    .await
    .unwrap();

    let input = format!(
        "{}/talos.tmp.{}.yaml",
        env!("CARGO_TARGET_TMPDIR"),
        vm.port
    );
    std::fs::write(&input, data).unwrap();
    dbg!(vm.port);

    Vm::wait_for_str(
        &vm.console_socket,
        "service[apid](Running): Started task apid",
        Duration::from_secs(10),
    )
    .unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;

    let time_to_installer = start.elapsed();
    let mut child = Command::new("talosctl")
        .args(["-e", &format!("127.0.0.2:{}", vm.port)])
        .args(["-n", &format!("127.0.0.2:{}", vm.port)])
        .args(["apply", "-i"])
        .args(["-f", &input])
        .spawn()
        .unwrap();

    child.wait().await.unwrap();

    wait_for_request(port).await.unwrap();
    let time_to_run_container = start.elapsed();

    child.kill().await.unwrap();

    Measurement {
        time_to_installer,
        time_to_run_container,
    }
}

#[tokio::main(flavor = "local")]
async fn main() {
    const NUM_ITER: usize = 1;

    // let mut cos = Vec::with_capacity(NUM_ITER);
    // for _ in 0..NUM_ITER {
    //     let data = cos_install().await;
    //     cos.push(data);
    // }

    let mut talos = Vec::with_capacity(NUM_ITER);
    for _ in 0..NUM_ITER {
        let data = talos_install().await;
        talos.push(data);
    }

    // dbg!(cos);
    dbg!(talos);
}

/*
qemu-system-x86_64 \
  -m 2048 \
  -smp 4 \
  -cpu host \
  -enable-kvm \
  -cdrom ./nocloud-amd64.iso \
  -netdev user,id=net0,hostfwd=tcp::50000-:50000 \
  -device e1000,netdev=net0 \
  -nographic \
  -serial mon:stdio

  qemu-system-x86_64 \
  -m 2048 \
  -cpu host \
  -enable-kvm \
  -cdrom ./nocloud-amd64.iso \
  -netdev user,id=net0,hostfwd=tcp::50000-:50000 \
  -device e1000,netdev=net0 \
  -nographic
*/
