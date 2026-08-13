use std::time::Duration;

use anyhow::Result;
use cosc::{CosClient, Key, Resource, SubResourceCreate, Value};

use crate::Vm;

pub struct CosVm {
    pub vm: Vm,
    pub client: CosClient,
    pub time_to_kernel: Duration,
    pub time_to_init: Duration,
    pub time_to_supervisor: Duration,
    pub time_to_reconcile: Duration,
    pub time_to_dhcp: Duration,
}

impl CosVm {
    #[expect(clippy::unwrap_used, reason = "API port should always exist")]
    pub async fn new(
        tmp: Option<&str>,
        disk: Option<u16>,
        mut ports: Vec<u16>,
    ) -> Result<Self> {
        ports.push(50000);
        let iso = std::env::var("E2E_DISK_IMAGE")?;
        let vm = Vm::new(tmp, &iso, disk, 256, ports).await?;

        let (
            time_to_kernel,
            time_to_init,
            time_to_supervisor,
            time_to_reconcile,
            time_to_dhcp,
        ) = Self::wait_for_init(&vm).await?;

        let client = CosClient::new(
            &format!("http://127.0.0.1:{}", vm.get_port(50000).unwrap()),
            None,
        )?;

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

    #[expect(clippy::unwrap_used, reason = "API port should always exist")]
    pub fn api_port(&self) -> u16 {
        self.vm.get_port(50000).unwrap()
    }

    pub fn get_port(&self, target: u16) -> Option<u16> {
        self.vm.get_port(target)
    }

    pub async fn wait_for_init(
        vm: &Vm,
    ) -> Result<(Duration, Duration, Duration, Duration, Duration)> {
        Vm::wait_for_str(
            &vm.console_socket,
            "Linux version",
            Duration::from_secs(10),
        )
        .await?;
        let time_to_kernel = vm.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "Run /init as init process",
            Duration::from_secs(10),
        )
        .await?;
        let time_to_init = vm.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "/bin/supervisor",
            Duration::from_secs(10),
        )
        .await?;
        let time_to_supervisor = vm.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "attempting to reconcile",
            Duration::from_secs(10),
        )
        .await?;
        let time_to_reconcile = vm.elapsed();

        Vm::wait_for_str(
            &vm.console_socket,
            "reconciled resource status=Ready key=network:route/eth0-dhcp",
            Duration::from_secs(10),
        )
        .await?;
        let time_to_dhcp = vm.elapsed();

        Ok((
            time_to_kernel,
            time_to_init,
            time_to_supervisor,
            time_to_reconcile,
            time_to_dhcp,
        ))
    }

    pub async fn wait_for_str(&self, pattern: &str) -> Result<()> {
        Vm::wait_for_str(
            &self.vm.console_socket,
            pattern,
            Duration::from_secs(60),
        )
        .await
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.vm.kill().await
    }

    pub async fn reboot(
        &mut self,
    ) -> Result<(Duration, Duration, Duration, Duration, Duration)> {
        self.vm.reboot().await?;
        let res = Self::wait_for_init(&self.vm).await?;
        let client = CosClient::new(
            &format!("http://127.0.0.1:{}", self.api_port()),
            None,
        )?;

        self.client = client;
        Ok(res)
    }

    pub async fn reconcile(&mut self, key: &Key) -> Result<()> {
        self.client.resources_reconcile_now(key).await
    }

    pub async fn push(
        &mut self,
        configs: &[SubResourceCreate<Value>],
    ) -> Result<()> {
        self.client.config_push(configs).await
    }

    pub async fn list(&mut self) -> Result<Vec<Resource>> {
        self.client.resources_list().await
    }

    pub async fn get_resource(&mut self, key: &Key) -> Result<Resource> {
        self.client.resources_get(key).await
    }

    pub async fn push_str(&mut self, s: &str) -> Result<()> {
        self.client.config_push_str(s).await
    }

    pub fn set_password(&mut self, password: Option<String>) {
        self.client.set_password(password);
    }

    pub fn elapsed(&self) -> Duration {
        self.vm.elapsed()
    }
}
