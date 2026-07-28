#![feature(exit_status_error)]

use std::io::{BufRead as _, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use cosc::{CosClient, Key, Resource, SubResourceCreate, Value};
use regex::Regex;
use tempfile::{TempDir, tempdir_in};
use tokio::process::{Child, Command};

pub struct Vm {
    tmpdir: TempDir,
    child: Child,
    disk: PathBuf,
    console_socket: PathBuf,
    qmp_socket: PathBuf,
    client: CosClient,
}

fn random_port() -> u16 {
    rand::random_range(32768..60999)
}

impl Vm {
    pub async fn try_start() -> Result<Self> {
        let vm = Self::try_create().await?;
        Self::wait_for_str(
            &vm.console_socket,
            "reconciled resource status=Ready key=network:route/eth0-dhcp",
            Duration::from_secs(10),
        )?;

        Ok(vm)
    }

    async fn try_create() -> Result<Self> {
        let iso = std::env::var("E2E_DISK_IMAGE")?;
        let mut tmpdir = tempdir_in(env!("CARGO_TARGET_TMPDIR"))?;
        tmpdir.disable_cleanup(true);
        let disk = tmpdir.path().join("disk.img");
        let console_socket = tmpdir.path().join("console.sock");
        let qmp_socket = tmpdir.path().join("qmp.sock");

        let mut child = Command::new("dd")
            .args([
                "if=/dev/zero",
                &format!("of={}", disk.display()),
                "bs=512M",
                "count=1",
            ])
            .spawn()?;
        let status = child.wait().await?;
        status.exit_ok()?;

        let port = random_port();
        let child = Command::new("qemu-system-x86_64")
            .args(["-enable-kvm"])
            .args(["-cdrom", &iso])
            .args(["-cpu", "host"])
            .args(["-m", "256"])
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
            .args([
                "-netdev",
                &format!("user,id=net0,hostfwd=tcp::{port}-:50000"),
            ])
            .args(["-device", "virtio-net-pci,netdev=net0"])
            .spawn()?;

        let client = CosClient::new(&format!("http://127.0.0.1:{port}"), None)?;

        Ok(Self {
            tmpdir,
            child,
            disk,
            console_socket,
            qmp_socket,
            client,
        })
    }

    fn wait_for_str(
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

    pub async fn reconcile(&mut self, key: &Key) -> Result<()> {
        self.client.reconcile(key).await
    }

    pub async fn push(
        &mut self,
        configs: &[SubResourceCreate<Value>],
    ) -> Result<()> {
        self.client.push(configs).await
    }

    pub async fn list(&mut self) -> Result<Vec<Resource>> {
        self.client.list().await
    }

    pub async fn get_resource(&mut self, key: &Key) -> Result<Resource> {
        self.client.get_resource(key).await
    }

    pub async fn push_str(&mut self, s: &str) -> Result<()> {
        self.client.push_str(s).await
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.client.set_password(password);
    }
}

impl Drop for Vm {
    #[expect(clippy::unwrap_used, reason = "we are running tests")]
    fn drop(&mut self) {
        self.child.start_kill().unwrap();
    }
}

#[cfg(test)]
mod validation {
    use serde_json::json;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio::net::TcpListener;

    use super::*;

    async fn wait_for_request(port: u16) -> Result<()> {
        let listener =
            TcpListener::bind(&format!("0.0.0.0:{port}")).await.unwrap();

        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = [0_u8; 1024];
            let _ = socket.read(&mut buf).await;

            let response = "HTTP/1.1 200 OK\r\nContent-Type: \
                            text/plain\r\nContent-Length: 16\r\nConnection: \
                            close\r\n\r\nhello from VM\n";

            socket.write_all(response.as_bytes()).await.unwrap();
        });

        Ok(())
    }

    #[tokio::test]
    async fn starts() {
        let _vm = Vm::try_start().await.unwrap();
    }

    #[tokio::test]
    async fn get_route() {
        let mut vm = Vm::try_start().await.unwrap();
        let resource = vm
            .get_resource(&Key {
                schema: "network:route".to_owned(),
                name: Some("eth0-dhcp".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(
            resource.spec,
            json!({
                "ipv4": {
                    "destination": "0.0.0.0",
                    "gateway": "10.0.2.2",
                    "prefix_len": 0,
                    "parent": "eth0-dhcp"
                }
            })
        );
    }

    #[tokio::test]
    async fn list_resources() {
        let mut vm = Vm::try_start().await.unwrap();
        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 7);
    }

    #[tokio::test]
    async fn create_config() {
        let port = random_port();
        let data = include_str!("./data/create-container.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = Vm::try_start().await.unwrap();
        let () = vm.push_str(&data).await.unwrap();
        vm.set_password(Some("hepia2026demo".to_owned()));

        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 10);
    }

    #[tokio::test]
    async fn create_container() {
        let port = random_port();
        let data = include_str!("./data/create-container.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = Vm::try_start().await.unwrap();
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "TODO"]
    async fn create_delete_container() {
        let port = random_port();
        let data = include_str!("./data/create-delete-container--create.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = Vm::try_start().await.unwrap();
        let () = vm.push_str(&data).await.unwrap();
        vm.set_password(Some("hepia2026demo".to_owned()));
        wait_for_request(port).await.unwrap();

        let data = include_str!("./data/create-delete-container--delete.yaml");
        let () = vm.push_str(data).await.unwrap();

        let resources = vm.list().await.unwrap();
        dbg!(&resources);
        assert_eq!(resources.len(), 7);
    }
}
