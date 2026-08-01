#![allow(
    clippy::unwrap_used,
    reason = "This crate is intended to be used only from tests."
)]

use std::io::{Read as _, Write as _};
use std::path::Path;

pub use isolation_macros::isolate;
use linux_utils::{SpecialFs, mount_special};
use nix::sched::{CloneFlags, clone};
use nix::sys::prctl;
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{getgid, getuid, pipe};
use rustix::mount::{MountFlags, MountPropagationFlags, mount_change};
use rustix::process::chroot;

const STACK_SIZE: usize = 1024 * 1024;

// The code bellow is inspired from https://github.com/canndrew/netsim/blob/1766dee89256f561df99e8f72d27ce75e45bd1cf/src/namespace.rs

/// Run a function as root in an isolated network namespace and an empty `/`.
///
/// # SHOULD ONLY BE USED IN INTEGRATIONS TESTS
pub fn namespaced<Root, F>(root: Root, f: F)
where
    Root: AsRef<Path>,
    F: FnOnce() + Send + std::panic::UnwindSafe + 'static,
{
    let mut stack = vec![0_u8; STACK_SIZE];
    let (panic_r, panic_w) = pipe().unwrap();

    let uid = getuid();
    let gid = getgid();

    let tmpdir = tempfile::tempdir_in(root).unwrap();
    let tmpdir_path = tmpdir.path();

    let mut f = Some(f);
    let mut panic_w = Some(panic_w);

    // SAFETY: TODO
    let child_pid = unsafe {
        clone(
            Box::new(move || {
                let panic_w = panic_w.take().unwrap();
                let writer = std::fs::File::from(panic_w);

                std::panic::set_hook(Box::new(move |info| {
                    let mut w = &writer;
                    let _ = writeln!(w, "{info}");
                    std::process::abort();
                }));

                let f = f.take().unwrap();
                let result = std::panic::catch_unwind(
                    std::panic::AssertUnwindSafe(move || {
                        prctl::set_pdeathsig(Signal::SIGTERM).unwrap();
                        std::fs::write(
                            "/proc/self/uid_map",
                            format!("0 {uid} 1\n"),
                        )
                        .unwrap();
                        std::fs::write("/proc/self/setgroups", "deny").unwrap();
                        std::fs::write(
                            "/proc/self/gid_map",
                            format!("0 {gid} 1\n"),
                        )
                        .unwrap();

                        mount_change(
                            "/",
                            MountPropagationFlags::DOWNSTREAM
                                | MountPropagationFlags::REC,
                        )
                        .unwrap();

                        let tmpdir = tmpdir_path.to_path_buf();
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
                        std::env::set_current_dir("/").unwrap();

                        let () = std::thread::spawn(move || {
                            let () = std::panic::catch_unwind(
                                std::panic::AssertUnwindSafe(f),
                            )
                            .unwrap();
                        })
                        .join()
                        .unwrap();
                    }),
                );

                #[expect(clippy::exit, reason = "TODO")]
                match result {
                    Ok(()) => std::process::exit(0),
                    Err(_err) => std::process::exit(1),
                }
            }),
            &mut stack,
            CloneFlags::CLONE_NEWCGROUP
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWNET
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWUSER
                | CloneFlags::CLONE_NEWUTS,
            #[expect(
                clippy::as_conversions,
                reason = "API expects an i32 and Signal is #[repr(i32)]"
            )]
            Some(Signal::SIGCHLD as i32),
        )
        .unwrap()
    };

    let status = waitpid(child_pid, None).unwrap();
    tmpdir.close().unwrap();

    #[expect(clippy::panic, reason = "TODO")]
    let WaitStatus::Exited(_, 0) = status else {
        let mut reader = std::fs::File::from(panic_r);
        let mut panic_msg = String::new();
        reader.read_to_string(&mut panic_msg).unwrap();

        panic!("{panic_msg}")
    };
}
