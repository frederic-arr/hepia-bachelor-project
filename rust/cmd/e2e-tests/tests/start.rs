use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use tempfile::{TempDir, tempdir_in};
// use tokio::process::{Child, Command};

pub struct Vm {
    tmpdir: TempDir,
    child: Child,
    disk: PathBuf,
    console_socket: PathBuf,
    qmp_socket: PathBuf,
}

impl Vm {
    #[must_use]
    pub fn start() -> Self {
        let vm = Self::create();
        Self::wait_for_str(
            &vm.console_socket,
            "api listening on 0.0.0.0:50000",
            Duration::from_secs(10),
        )
        .unwrap();
        vm
    }

    #[must_use]
    pub fn create() -> Self {
        let mut iso = std::env::var("E2E_DISK_IMAGE").unwrap();
        let mut tmpdir = tempdir_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
        tmpdir.disable_cleanup(true);
        let disk = tmpdir.path().join("disk.img");
        let console_socket = tmpdir.path().join("console.sock");
        let qmp_socket = tmpdir.path().join("qmp.sock");

        let mut child = Command::new("dd")
            .args([
                "if=/dev/zero",
                &format!("of={}", disk.display()),
                "bs=1M",
                "count=512",
            ])
            .spawn()
            .unwrap();
        let status = child.wait().unwrap();
        assert!(status.success(), "should be able to create disk");

        let mut child = Command::new("mkfs.ext4").arg(&disk).spawn().unwrap();
        let status = child.wait().unwrap();
        assert!(status.success(), "should be able to format disk");

        let child = Command::new("qemu-system-x86_64")
            .args(["-enable-kvm"])
            .args(["-cdrom", &iso])
            .args(["-cpu", "host"])
            .args(["-m", "512"])
            .args(["-nographic", "-no-reboot"])
            .args([
                "-drive",
                &format!("file={},format=raw,if=virtio", disk.display()),
            ])
            .args([
                "-chardev",
                &format!(
                    "socket,id=serial0,path={},server=on,wait=off",
                    console_socket.display()
                ),
            ])
            .args(["-serial", "chardev:serial0"])
            .args([
                "-qmp",
                &format!("unix:{},server,wait=off", qmp_socket.display()),
            ])
            .args(["-netdev", "user,id=net0,hostfwd=tcp::1234-:1234"])
            .args(["-device", "virtio-net-pci,netdev=net0"])
            .spawn()
            .unwrap();

        Self {
            tmpdir,
            child,
            disk,
            console_socket,
            qmp_socket,
        }
    }

    pub fn wait_for_str(
        socket: &PathBuf,
        pattern: &str,
        timeout: Duration,
    ) -> Result<(), String> {
        let stream = {
            let deadline = Instant::now() + timeout;
            loop {
                match UnixStream::connect(socket) {
                    Ok(s) => break s,
                    Err(_) if Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => return Err(format!("Connect failed: {e}")),
                }
            }
        };

        stream.set_read_timeout(Some(timeout)).unwrap();
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if line.contains(pattern) {
                return Ok(());
            }
        }
        Err(format!(
            "Pattern '{pattern}' not found before timeout"
        ))
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

#[test]
fn bob() {
    let vm = Vm::start();
}
