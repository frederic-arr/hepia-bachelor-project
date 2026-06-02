use std::ffi::CString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use loopdev::LoopControl;
use rustix::mount::{MountFlags, UnmountFlags, mount, mount_move, unmount};
use rustix::process::{chdir, chroot};

// https://github.com/cleverca22/not-os
// https://artemis.sh/2023/03/07/nixos-early-boot-running-from-ram.html
// https://github.com/util-linux/util-linux/blob/master/sys-utils/switch_root.c
fn switch_root(new_root: &str, init: &str) -> std::io::Result<()> {
    // tracing::info!("switch_root to {new_root} and execv {init}");
    let umounts = ["/dev", "/proc", "/sys", "/run"];
    for umount in umounts {
        // tracing::debug!("moving {umount}");
        if !std::fs::exists(umount).unwrap() {
            // tracing::debug!("{umount} does not exist, skipping");
            continue;
        }

        let new_mount = format!("{new_root}{umount}");

        // TODO: It's a RO fs
        // unmount(umount, UnmountFlags::DETACH);
        // std::fs::create_dir(&new_mount).unwrap();
        // mount_move(umount, new_mount.as_str()).unwrap();
    }

    chdir(new_root).unwrap();
    mount_move(new_root, "/").unwrap();
    chroot(".").unwrap();
    chdir("/").unwrap();

    Err(Command::new(init).exec())
}

fn mount_squashfs(image: &str, target: &str) -> std::io::Result<()> {
    let lc = LoopControl::open()?;
    let ld = lc.next_free()?;
    ld.attach_file(image)?;

    let opts = CString::new("ro")?;
    mount(
        ld.path().unwrap(),
        target,
        "squashfs",
        MountFlags::empty(),
        Some(opts.as_c_str()),
    )?;

    Ok(())
}

fn mount_pseudofs() {
    let opts = CString::new("mode=0755").unwrap();
    mount(
        "devtmpfs",
        "/dev",
        "devtmpfs",
        MountFlags::NOSUID | MountFlags::STRICTATIME,
        Some(opts.as_c_str()),
    )
    .unwrap();

    std::fs::create_dir("/proc").unwrap();
    mount(
        "proc",
        "/proc",
        "proc",
        MountFlags::NOSUID | MountFlags::NOEXEC | MountFlags::NODEV,
        None,
    )
    .unwrap();

    std::fs::create_dir("/sys").unwrap();
    mount(
        "sysfs",
        "/sys",
        "sysfs",
        MountFlags::NOSUID | MountFlags::NOEXEC | MountFlags::RELATIME,
        None,
    )
    .unwrap();
}

fn main() {
    mount_pseudofs();

    println!("=== START ===");
    let paths = std::fs::read_dir("./").unwrap();
    for path in paths {
        println!("{}", path.unwrap().path().display());
    }

    println!("=== MOUNT ===");
    std::fs::create_dir("/mnt").unwrap();
    mount_squashfs("/root.squashfs", "/mnt").unwrap();

    let paths = std::fs::read_dir("./mnt").unwrap();
    for path in paths {
        println!("{}", path.unwrap().path().display());
    }

    switch_root("/mnt", "/bin/supervisor").unwrap();
    let paths = std::fs::read_dir("./").unwrap();
    for path in paths {
        println!("{}", path.unwrap().path().display());
    }

    loop {
        std::thread::park();
    }
}
