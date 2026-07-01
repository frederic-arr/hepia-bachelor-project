use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use linux_utils::{
    SpecialFs,
    attach_loop,
    mount_iso,
    mount_overlayfs,
    mount_special,
    mount_squashfs,
};
use rustix::mount::{MountFlags, UnmountFlags, mount, mount_move, unmount};
use rustix::process::{chdir, chroot};

// https://github.com/cleverca22/not-os
// https://artemis.sh/2023/03/07/nixos-early-boot-running-from-ram.html
// https://github.com/util-linux/util-linux/blob/master/sys-utils/switch_root.c
fn switch_root<NewRoot>(new_root: NewRoot, init: &str) -> std::io::Result<()>
where
    NewRoot: AsRef<Path>,
{
    tracing::info!("switch_root to {}", new_root.as_ref().display());

    rustix::mount::mount_move("/dev", new_root.as_ref().join("dev")).unwrap();
    rustix::mount::mount_move("/proc", new_root.as_ref().join("proc")).unwrap();

    chdir(new_root.as_ref()).unwrap();
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
    mount_iso("/mnt/iso", "/dev/sr0", MountFlags::empty(), &[]).unwrap();

    let ld = attach_loop("/mnt/iso/root.squashfs").unwrap();
    mount_squashfs(
        "/mnt/rootfs",
        ld.path().unwrap(),
        MountFlags::empty(),
        &[],
    )
    .unwrap();

    mount_overlayfs(
        &["/mnt/rootfs"],
        Some(("/mnt/upper", "/mnt/work")),
        // None::<(PathBuf, PathBuf)>,
        "/mnt/merged",
        MountFlags::empty(),
        &[],
    )
    .unwrap();
    switch_root("/mnt/merged", "/bin/supervisor").unwrap();
    unreachable!();
}
