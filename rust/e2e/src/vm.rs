use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use qapi::futures::QmpStreamTokio;
use regex::Regex;
use tempfile::{TempDir, tempdir, tempdir_in};
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::net::UnixStream;
use tokio::process::{Child, Command};

use crate::random_port;

#[derive(Debug)]
pub struct Vm {
    pub iso: String,
    pub memory_size: u16,
    pub tmpdir: TempDir,
    pub child: Child,
    pub disk: Option<PathBuf>,
    pub console_socket: PathBuf,
    pub qmp_socket: PathBuf,
    pub start: Instant,
    pub ports: Vec<(u16, u16)>,
}

impl Vm {
    pub async fn new(
        tmp: Option<&str>,
        image: &str,
        disk_size: Option<u16>,
        memory_size: u16,
        ports: Vec<u16>,
    ) -> Result<Self> {
        let iso = image;
        let tmpdir = match tmp {
            Some(p) => tempdir_in(p)?,
            None => tempdir()?,
        };

        let console_socket = tmpdir.path().join("console.sock");
        let qmp_socket = tmpdir.path().join("qmp.sock");
        let ports = ports
            .iter()
            .map(|dest| (random_port(), *dest))
            .collect::<Vec<_>>();

        let disk = if let Some(disk_size) = disk_size {
            let disk = tmpdir.path().join("disk.img");
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
            Some(disk)
        } else {
            None
        };

        let start = Instant::now();
        let child = Self::create_qemu(
            iso,
            disk.as_ref(),
            memory_size,
            &console_socket,
            &qmp_socket,
            &ports,
        )
        .await?;

        Ok(Self {
            iso: iso.to_owned(),
            memory_size,
            tmpdir,
            child,
            disk,
            console_socket,
            qmp_socket,
            start,
            ports,
        })
    }

    #[must_use]
    pub fn get_port(&self, target: u16) -> Option<u16> {
        self.ports
            .iter()
            .find_map(|(src, dest)| (*dest == target).then_some(*src))
    }

    async fn create_qemu(
        iso: &str,
        disk: Option<&PathBuf>,
        memory_size: u16,
        console_socket: &Path,
        qmp_socket: &Path,
        ports: &[(u16, u16)],
    ) -> Result<Child> {
        let ports = ports
            .iter()
            .map(|(src, dest)| format!("hostfwd=tcp::{src}-:{dest}"))
            .collect::<Vec<_>>()
            .join(",");

        let mut cmd = Command::new("qemu-system-x86_64");
        cmd.args(["-enable-kvm"])
            .args(["-cdrom", iso])
            .args(["-cpu", "host"])
            .args(["-smp", "4"])
            .args(["-m", &memory_size.to_string()])
            .args(["-nographic"])
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
            .args(["-netdev", &format!("user,id=net0,{ports}")])
            .args(["-device", "virtio-net-pci,netdev=net0"])
            .kill_on_drop(true);

        if let Some(disk) = disk {
            cmd.args([
                "-drive",
                &format!("file={},format=raw,if=virtio", disk.display()),
            ]);
        }

        cmd.spawn().map_err(Into::into)
    }

    pub async fn wait_for_str(
        socket: &PathBuf,
        pattern: &str,
        timeout: Duration,
    ) -> Result<()> {
        let stream = {
            let deadline = Instant::now() + timeout;
            loop {
                match UnixStream::connect(socket).await {
                    Ok(s) => break s,
                    Err(_) if Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => bail!("Connect failed: {e}"),
                }
            }
        };

        let reader = BufReader::new(stream);

        let re = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")
            .map_err(|e| anyhow!("failed to create regex: {e}"))?;

        let mut lines = reader.lines();
        loop {
            let ln = lines.next_line().await;
            let Some(line) = ln? else { break };
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

    pub async fn reboot(&mut self) -> Result<()> {
        let stream = QmpStreamTokio::open_uds(&self.qmp_socket).await?;
        let stream = stream.negotiate().await?;
        let (qmp, handle) = stream.spawn_tokio();
        qmp.execute(qapi::qmp::system_reset {}).await?;
        drop(qmp);
        handle.await?;
        Ok(())
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}
