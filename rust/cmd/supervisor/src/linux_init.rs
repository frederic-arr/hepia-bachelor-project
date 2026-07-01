use linux_utils::{SpecialFs, mount_special};
use rustix::mount::{MountFlags, mount};

const INIT_PID: u32 = 1;

pub fn linux_init() {
    let pid = std::process::id();

    if pid != INIT_PID {
        tracing::error!("PID mismatch, expected PID {INIT_PID}, got PID {pid}");
        panic!("not PID1");
    }

    tracing::trace!("creating rootfs structure");
    create_rfs().unwrap();
}

const MFSEC: MountFlags = MountFlags::from_bits_truncate(
    MountFlags::NOSUID.bits()
        | MountFlags::NODEV.bits()
        | MountFlags::NOEXEC.bits()
        | MountFlags::RELATIME.bits(),
);

#[expect(clippy::unnecessary_wraps, reason = "will be dealt with later")]
fn create_rfs() -> std::io::Result<()> {
    mount_special(&SpecialFs::Sys, "/sys", MFSEC, &[]).unwrap();
    mount_special(&SpecialFs::Tmp, "/tmp", MFSEC, &[]).unwrap();
    mount_special(&SpecialFs::Tmp, "/run", MFSEC, &[]).unwrap();
    mount_special(&SpecialFs::Tmp, "/dev/shm", MFSEC, &[]).unwrap();

    mount_special(
        &SpecialFs::DevPts,
        "/dev/pts",
        MountFlags::NOSUID | MountFlags::NOEXEC | MountFlags::RELATIME,
        &["mode=620"],
    )
    .unwrap();

    mount_special(
        &SpecialFs::Hugetlbfs,
        "/dev/hugepages",
        MountFlags::NOSUID | MountFlags::NODEV | MountFlags::RELATIME,
        &["pagesize=2M"],
    )
    .unwrap();

    mount_special(
        &SpecialFs::Trace,
        "/sys/kernel/tracing",
        MFSEC,
        &[],
    )
    .unwrap();

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Debug,
    //     "/sys/kernel/debug",
    //     MFSEC,
    //     &[],
    // )
    // .unwrap();

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Security,
    //     "/sys/kernel/security",
    //     MFSEC,
    //     &[],
    // )
    // .unwrap();

    // TODO: Enable Kernel flag
    // mount_special(
    //     SpecialFs::Bpf,
    //     "/sys/fs/bpf",
    //     MountFlags::RELATIME,
    //     &[],
    // )
    // .unwrap();

    mount_special(&SpecialFs::Cgroup2, "/sys/fs/cgroup", MFSEC, &[]).unwrap();

    let tmpfs = [
        // "/etc",
        "/home", "/media", "/mnt", "/opt", "/run", "/sbin", "/srv", "/tmp",
        "/usr", // "/var",
    ];

    mount(
        "/dev/vda",
        "/var",
        "ext4",
        MountFlags::empty(),
        None,
    )
    .unwrap();

    for target in tmpfs {
        mount_special(&SpecialFs::Tmp, target, MFSEC, &[]).unwrap();
    }

    let dirs = [
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
        std::fs::create_dir_all(dir).unwrap();
    }

    Ok(())
}
