#![feature(never_type)]

mod linux_init;

use std::process::Stdio;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::process::{ChildStdout, Command};
use tokio::task::JoinSet;

use crate::linux_init::linux_init;

async fn wait_for_line(mut f: Option<ChildStdout>, ln: &str) -> Result<()> {
    if let Some(stdout) = f.take() {
        let mut lines = BufReader::new(stdout).lines();

        #[expect(clippy::print_stdout, reason = "TODO")]
        while let Some(line) = lines.next_line().await? {
            println!("{line}");

            if line.contains(ln) {
                break;
            }
        }

        Ok(())
    } else {
        Ok(())
    }
}

#[tokio::main(flavor = "local")]
async fn main() -> Result<!> {
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

    busybox.wait().await?;

    let netctl = Command::new("/bin/network-controller")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let sysctl = Command::new("/bin/system-controller")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let conctl = Command::new("/bin/container-controller")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut set = JoinSet::new();
    set.spawn(wait_for_line(netctl.stdout, "listening"));
    set.spawn(wait_for_line(sysctl.stdout, "listening"));
    set.spawn(wait_for_line(conctl.stdout, "listening"));

    set.join_all().await;
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

    busybox.wait().await?;
    statemgr.wait().await?;

    loop {
        std::thread::park();
    }
}
