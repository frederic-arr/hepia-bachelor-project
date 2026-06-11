use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use linux_utils::{SpecialFs, mount_special, mount_squashfs};
use rustix::mount::{MountFlags, UnmountFlags, mount, mount_move, unmount};
use rustix::process::{chdir, chroot};

// https://github.com/cleverca22/not-os
// https://artemis.sh/2023/03/07/nixos-early-boot-running-from-ram.html
// https://github.com/util-linux/util-linux/blob/master/sys-utils/switch_root.c
fn switch_root(new_root: &str, init: &str) -> std::io::Result<()> {
    tracing::info!("switch_root to {new_root}");

    rustix::mount::mount_move("/dev", "/mnt/dev").unwrap();
    rustix::mount::mount_move("/proc", "/mnt/proc").unwrap();

    chdir(new_root).unwrap();
    chroot(".").unwrap();
    chdir("/").unwrap();

    tracing::info!("exec {init}");
    Err(Command::new(init).exec())
}

fn mount_pseudofs() -> std::io::Result<()> {
    mount_special(
        &SpecialFs::Dev,
        "/dev",
        MountFlags::NOSUID | MountFlags::RELATIME,
        &["mode=755"],
    )?;

    mount_special(
        &SpecialFs::Proc,
        "/proc",
        MountFlags::NOSUID
            | MountFlags::NOEXEC
            | MountFlags::NODEV
            | MountFlags::RELATIME,
        &[],
    )
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    assert_eq!(std::process::id(), 1, "/init must be run as PID1");
    mount_pseudofs().unwrap();
    mount_squashfs("/mnt", "/root.squashfs", MountFlags::empty(), &[]).unwrap();
    switch_root("/mnt", "/bin/supervisor").unwrap();
    unreachable!();
}
