use std::io::{BufRead as _, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use tempfile::{TempDir, tempdir, tempdir_in};
use tokio::process::{Child, Command};

use crate::random_port;

pub struct Vm {
    pub tmpdir: TempDir,
    pub child: Child,
    pub disk: Option<PathBuf>,
    pub console_socket: PathBuf,
    pub qmp_socket: PathBuf,
    pub port: u16,
}

impl Vm {
    pub async fn new(
        tmp: Option<&str>,
        image: &str,
        target_port: u16,
        disk_size: Option<u16>,
        memory_size: u16,
    ) -> Result<Self> {
        let iso = image;
        let tmpdir = match tmp {
            Some(p) => tempdir_in(p)?,
            None => tempdir()?,
        };

        let disk = tmpdir.path().join("disk.img");
        let console_socket = tmpdir.path().join("console.sock");
        let qmp_socket = tmpdir.path().join("qmp.sock");

        let port = random_port();
        let mut cmd = Command::new("qemu-system-x86_64");
        cmd.args(["-enable-kvm"])
            .args(["-cdrom", iso])
            .args(["-cpu", "host"])
            .args(["-m", &memory_size.to_string()])
            .args(["-nographic", "-no-reboot"])
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
            .args([
                "-netdev",
                &format!("user,id=net0,hostfwd=tcp::{port}-:{target_port}"),
            ])
            .args(["-device", "virtio-net-pci,netdev=net0"]);

        if let Some(disk_size) = disk_size {
            let mut child = Command::new("dd")
                .args([
                    "if=/dev/zero",
                    &format!("of={}", disk.display()),
                    &format!("bs={disk_size}M"),
                    "count=1",
                ])
                .spawn()?;
            let status = child.wait().await?;
            status.exit_ok()?;

            cmd.args([
                "-drive",
                &format!("file={},format=raw,if=virtio", disk.display()),
            ]);
        }

        let child = cmd.spawn()?;

        Ok(Self {
            tmpdir,
            child,
            disk: disk_size.map(|_| disk),
            console_socket,
            qmp_socket,
            port,
        })
    }

    pub fn wait_for_str(
        socket: &PathBuf,
        pattern: &str,
        timeout: Duration,
    ) -> Result<()> {
        let stream = {
            let deadline = Instant::now() + timeout;
            loop {
                match UnixStream::connect(socket) {
                    Ok(s) => break s,
                    Err(_) if Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => bail!("Connect failed: {e}"),
                }
            }
        };

        stream.set_read_timeout(Some(timeout))?;
        let reader = BufReader::new(stream);

        let re = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")
            .map_err(|e| anyhow!("failed to create regex: {e}"))?;

        for line in reader.lines() {
            let line = line?;
            let line = re.replace_all(&line, "");
            if line.contains(pattern) {
                return Ok(());
            }
        }

        bail!("Pattern '{pattern}' not found before timeout")
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await.map_err(Into::into)
    }
}

impl Drop for Vm {
    #[expect(clippy::unwrap_used, reason = "we are running tests")]
    fn drop(&mut self) {
        self.child.start_kill().unwrap();
    }
}
