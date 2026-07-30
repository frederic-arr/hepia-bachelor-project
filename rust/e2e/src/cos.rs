use std::time::{Duration, Instant};

use anyhow::Result;
use cosc::{CosClient, Key, Resource, SubResourceCreate, Value};

use crate::Vm;

pub struct CosVm {
    vm: Vm,
    client: CosClient,
    pub time_to_kernel: Duration,
    pub time_to_init: Duration,
    pub time_to_supervisor: Duration,
    pub time_to_reconcile: Duration,
    pub time_to_dhcp: Duration,
}

impl CosVm {
    pub async fn new(tmp: Option<&str>, disk: Option<u16>) -> Result<Self> {
        let iso = std::env::var("E2E_DISK_IMAGE")?;
        let start = Instant::now();

        let vm = Vm::new(tmp, &iso, 50000, disk, 256).await?;

        Vm::wait_for_str(
            &vm.console_socket,
            "Linux version",
            Duration::from_secs(10),
        )?;
        let time_to_kernel = start.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "Run /init as init process",
            Duration::from_secs(10),
        )?;
        let time_to_init = start.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "/bin/supervisor",
            Duration::from_secs(10),
        )?;
        let time_to_supervisor = start.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "attempting to reconcile",
            Duration::from_secs(10),
        )?;
        let time_to_reconcile = start.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "reconciled resource status=Ready key=network:route/eth0-dhcp",
            Duration::from_secs(10),
        )?;
        let time_to_dhcp = start.elapsed();

        let client =
            CosClient::new(&format!("http://127.0.0.1:{}", vm.port), None)?;

        Ok(Self {
            vm,
            client,
            time_to_kernel,
            time_to_init,
            time_to_supervisor,
            time_to_reconcile,
            time_to_dhcp,
        })
    }

    pub async fn wait_for_str(&self, pattern: &str) -> Result<()> {
        Vm::wait_for_str(
            &self.vm.console_socket,
            pattern,
            Duration::from_secs(10),
        )
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.vm.kill().await
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
