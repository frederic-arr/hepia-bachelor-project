use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow, bail};
use linux_utils::{
    SpecialFs,
    attach_loop,
    get_boot_disk,
    is_maintenance,
    mount_iso,
    mount_overlayfs,
    mount_special,
    mount_squashfs,
};
use rustix::mount::{MountFlags, mount, mount_move};
use rustix::process::{chdir, chroot};

// https://github.com/cleverca22/not-os
// https://artemis.sh/2023/03/07/nixos-early-boot-running-from-ram.html
// https://github.com/util-linux/util-linux/blob/master/sys-utils/switch_root.c
fn switch_root<NewRoot>(new_root: NewRoot, init: &str) -> Result<()>
where
    NewRoot: AsRef<Path>,
{
    tracing::info!("switch_root to {}", new_root.as_ref().display());

    mount_move("/dev", new_root.as_ref().join("dev"))?;
    mount_move("/proc", new_root.as_ref().join("proc"))?;

    let old_root_fd = std::fs::File::open("/")?;
    chdir(new_root.as_ref())?;
    mount_move(new_root.as_ref(), "/")?;
    chroot(".")?;
    chdir("/")?;

    drop(old_root_fd);

    tracing::info!("exec {init}");
    Err(Command::new(init).exec().into())
}

fn mount_pseudofs() -> Result<()> {
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

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    if std::process::id() != 1 {
        bail!("/init must be run as PID1");
    }

    mount_pseudofs()?;
    if is_maintenance() {
        mount_iso("/mnt/iso", "/dev/sr0", MountFlags::empty(), &[])?;

        let ld = attach_loop("/mnt/iso/root.squashfs")?;
        mount_squashfs(
            "/mnt/rootfs",
            ld.path().ok_or_else(|| anyhow!("TODO"))?,
            MountFlags::empty(),
            &[],
        )?;
    } else if let Some(disk) = get_boot_disk() {
        std::fs::create_dir_all("/mnt/boot")?;
        std::fs::create_dir_all("/mnt/rootfs")?;
        mount(
            disk,
            "/mnt/boot",
            "vfat",
            MountFlags::empty(),
            None,
        )?;

        let ld = attach_loop("/mnt/boot/root.squashfs")?;
        mount_squashfs(
            "/mnt/rootfs",
            ld.path().ok_or_else(|| anyhow!("TODO"))?,
            MountFlags::empty(),
            &[],
        )?;
    }
    mount_overlayfs(
        &["/mnt/rootfs"],
        Some(("/mnt/upper", "/mnt/work")),
        "/mnt/merged",
        MountFlags::empty(),
        &[],
    )?;

    switch_root("/mnt/merged", "/bin/supervisor")?;
    unreachable!();
}
