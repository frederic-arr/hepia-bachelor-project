#![feature(never_type)]

mod linux_init;

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::linux_init::linux_init;

fn main() -> Result<!> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    linux_init()?;

    let mut busybox = Command::new("/bin/busybox")
        .arg("sh")
        .arg("-c")
        .arg("ip link set dev lo up")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    busybox.wait()?;

    let mut netctl = Command::new("/bin/network-controller")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut sysctl = Command::new("/bin/system-controller")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // let mut conctl = Command::new("/bin/container-controller")
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit())
    //     .spawn()?;

    thread::sleep(Duration::from_millis(100));

    let mut statemgr = Command::new("/bin/state-manager")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

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
        .spawn()?;

    busybox.wait()?;
    netctl.wait()?;
    sysctl.wait()?;
    // conctl.wait()?;
    statemgr.wait()?;

    #[expect(
        clippy::infinite_loop,
        reason = "this is the init and can never stop"
    )]
    loop {
        std::thread::park();
    }
}
