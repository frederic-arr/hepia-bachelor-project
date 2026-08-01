use std::fs::set_permissions;
use std::os::unix::fs::PermissionsExt;

use anyhow::{Result, bail};
use linux_utils::{SpecialFs, get_config_disk, get_data_disk, mount_special};
use rustix::mount::{MountFlags, mount};

const INIT_PID: u32 = 1;

pub fn linux_init() -> Result<()> {
    let pid = std::process::id();

    if pid != INIT_PID {
        tracing::error!("PID mismatch, expected PID {INIT_PID}, got PID {pid}");
        bail!("not PID1");
    }

    tracing::trace!("creating rootfs structure");
    create_rfs()
}

const MFSEC: MountFlags = MountFlags::from_bits_truncate(
    MountFlags::NOSUID.bits()
        | MountFlags::NODEV.bits()
        | MountFlags::NOEXEC.bits()
        | MountFlags::RELATIME.bits(),
);

fn create_rfs() -> Result<()> {
    mount_special(&SpecialFs::Sys, "/sys", MFSEC, &[])?;
    mount_special(&SpecialFs::Tmp, "/tmp", MFSEC, &[])?;
    mount_special(&SpecialFs::Tmp, "/run", MFSEC, &[])?;
    mount_special(&SpecialFs::Tmp, "/dev/shm", MFSEC, &[])?;

    mount_special(
        &SpecialFs::DevPts,
        "/dev/pts",
        MountFlags::NOSUID | MountFlags::NOEXEC | MountFlags::RELATIME,
        &["mode=620"],
    )?;

    mount_special(
        &SpecialFs::Hugetlbfs,
        "/dev/hugepages",
        MountFlags::NOSUID | MountFlags::NODEV | MountFlags::RELATIME,
        &["pagesize=2M"],
    )?;

    mount_special(
        &SpecialFs::Trace,
        "/sys/kernel/tracing",
        MFSEC,
        &[],
    )?;

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Debug,
    //     "/sys/kernel/debug",
    //     MFSEC,
    //     &[],
    // )
    // ?;

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Security,
    //     "/sys/kernel/security",
    //     MFSEC,
    //     &[],
    // )
    // ?;

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Bpf,
    //     "/sys/fs/bpf",
    //     MountFlags::RELATIME,
    //     &[],
    // )
    // ?;

    mount_special(&SpecialFs::Cgroup2, "/sys/fs/cgroup", MFSEC, &[])?;

    let mut tmpfs = vec![
        "/home", "/media", "/mnt", "/opt", "/run", "/sbin", "/srv", "/tmp",
        "/usr", // "/var",
    ];

    if let Some(disk) = get_config_disk() {
        std::fs::create_dir_all("/config")?;
        mount(disk, "/config", "vfat", MountFlags::empty(), None)?;
    }

    if let Some(disk) = get_data_disk() {
        std::fs::create_dir_all("/var")?;
        mount(disk, "/var", "ext4", MountFlags::empty(), None)?;
    } else {
        tmpfs.push("/var");
    }

    for target in tmpfs {
        mount_special(&SpecialFs::Tmp, target, MFSEC, &[])?;
    }

    let dirs = [
        "/etc/containers",
        "/var/lib/podman-data",
        "/var/lib/containers/storage/overlay/diff",
        "/etc/opt",
        "/usr/bin",
        "/usr/include",
        "/usr/lib",
        "/usr/libexec",
        "/usr/local",
        "/usr/sbin",
        "/usr/share",
        "/usr/src",
        "/var/account",
        "/var/cache",
        "/var/cache/fonts",
        "/var/cache/man",
        "/var/crash",
        "/var/games",
        "/var/lib",
        "/var/lib/color",
        "/var/lib/hwclock",
        "/var/lib/misc",
        "/var/lock",
        "/var/log",
        "/var/mail",
        "/var/opt",
        "/var/run",
        "/var/spool",
        "/var/spool/cron",
        "/var/spool/lpd",
        "/var/spool/rwho",
        "/var/tmp",
        "/var/typ",
    ];

    for dir in dirs {
        std::fs::create_dir_all(dir)?;
    }

    set_permissions("/var/tmp", PermissionsExt::from_mode(0o1777))?;

    Ok(())
}
