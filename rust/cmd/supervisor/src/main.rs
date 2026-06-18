mod linux_init;

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::linux_init::linux_init;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    linux_init();

    // println!("Hello from supervisor!");
    let mut busybox = Command::new("/bin/busybox")
        .arg("sh")
        .arg("-c")
        .arg("ip link set dev lo up")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    busybox.wait().unwrap();

    let mut conmgr = Command::new("/bin/container-manager")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let mut netmgr = Command::new("/bin/network-manager")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_secs(5));

    let mut sysmgr = Command::new("/bin/system-manager")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    // mkdir -p /sys/fs/cgroup/cpu
    // mkdir -p /sys/fs/cgroup/cpuacct
    // mkdir -p /sys/fs/cgroup/blkio
    // mkdir -p /sys/fs/cgroup/devices
    // mkdir -p /sys/fs/cgroup/freezer
    // mkdir -p /sys/fs/cgroup/pids
    // mount -t cgroup -o cpu cpu /sys/fs/cgroup/cpu
    // mount -t cgroup -o cpuacct cpuacct /sys/fs/cgroup/cpuacct
    // mount -t cgroup -o blkio blkio /sys/fs/cgroup/blkio
    // mount -t cgroup -o devices devices /sys/fs/cgroup/devices
    // mount -t cgroup -o freezer freezer /sys/fs/cgroup/freezer
    // mount -t cgroup -o pids pids /sys/fs/cgroup/pids

    let mut busybox = Command::new("/bin/busybox")
        .arg("sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    busybox.wait().unwrap();
    conmgr.wait().unwrap();
    sysmgr.wait().unwrap();
    netmgr.wait().unwrap();

    loop {
        std::thread::park();
    }
}
