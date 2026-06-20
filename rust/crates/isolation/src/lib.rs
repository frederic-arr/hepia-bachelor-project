#![cfg(unix)]

use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::path::Path;
use std::sync::{Arc, Barrier};
use std::time::Duration;

use linux_utils::{SpecialFs, mount_special};
use nix::sched::{CloneFlags, clone};
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{
    close,
    dup2_stderr,
    dup2_stdout,
    getgid,
    getuid,
    pipe,
    read,
    write,
};
use rustix::mount::{MountFlags, MountPropagationFlags, mount, mount_change};
use rustix::process::chroot;

/// Run a function as root in an isolated network namespace and an empty `/`.
///
/// # SHOULD ONLY BE USED IN INTEGRATIONS TESTS
pub fn namespaced<Root, F, Fut>(root: Root, f: F)
where
    Root: AsRef<Path>,
    F: FnOnce() -> Fut + std::panic::UnwindSafe,
    Fut: std::future::Future<Output = ()>,
{
    let (sync_r, sync_w) = pipe().unwrap();
    let sync_r = sync_r.into_raw_fd();
    let sync_w = sync_w.into_raw_fd();

    let mut tmpdir = tempfile::tempdir_in(root).unwrap();
    let tmpdir_path = tmpdir.path();

    const STACK_SIZE: usize = 1024 * 1024;
    let mut stack = vec![0u8; STACK_SIZE];

    let mut f = Some(f);
    let child_pid = unsafe {
        clone(
            Box::new(move || {
                let result = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(|| {
                        let tmpdir = tmpdir_path.to_owned();
                        let mut buf = [0u8; 1];
                        read(File::from_raw_fd(sync_r), &mut buf).unwrap();
                        close(File::from_raw_fd(sync_r));

                        let f = f.take().unwrap();

                        mount_change(
                            "/",
                            MountPropagationFlags::DOWNSTREAM
                                | MountPropagationFlags::REC,
                        );

                        std::fs::create_dir(tmpdir.join("proc")).unwrap();
                        mount_special(
                            &SpecialFs::Proc,
                            tmpdir.join("proc"),
                            MountFlags::NOSUID
                                | MountFlags::NOEXEC
                                | MountFlags::NODEV
                                | MountFlags::RELATIME,
                            &[],
                        )
                        .unwrap();
                        chroot(tmpdir).unwrap();

                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            f().await;
                        })
                    }),
                );
                match result {
                    Ok(_) => 0,
                    Err(_) => 1,
                }
            }),
            &mut stack,
            CloneFlags::CLONE_NEWUSER
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWNET,
            Some(Signal::SIGCHLD as i32),
        )
    }
    .expect("clone failed");

    let uid = getuid();
    let gid = getgid();

    std::fs::write(
        format!("/proc/{child_pid}/uid_map"),
        format!("0 {} 1\n", uid),
    )
    .unwrap();
    std::fs::write(format!("/proc/{child_pid}/setgroups"), "deny").unwrap();
    std::fs::write(
        format!("/proc/{child_pid}/gid_map"),
        format!("0 {} 1\n", gid),
    )
    .unwrap();

    unsafe {
        write(File::from_raw_fd(sync_w), &[1]).unwrap();
        close(File::from_raw_fd(sync_w));
    }

    let status = waitpid(child_pid, None).unwrap();
    if let WaitStatus::Exited(_, 0) = status {
        return;
    }

    tmpdir.disable_cleanup(true);
    panic!()
}

pub use isolation_macros::isolate;
