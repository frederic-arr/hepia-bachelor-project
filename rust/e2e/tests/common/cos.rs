use std::time::Duration;

use anyhow::Result;
use cosc::{CosClient, Key, Resource, SubResourceCreate, Value};

use crate::common::Vm;

pub struct CosVm {
    vm: Vm,
    client: CosClient,
}

impl CosVm {
    pub async fn new() -> Result<Self> {
        let iso = std::env::var("E2E_DISK_IMAGE")?;
        let vm = Vm::new(&iso, 50000, 512, 256).await?;

        Vm::wait_for_str(
            &vm.console_socket,
            "reconciled resource status=Ready key=network:route/eth0-dhcp",
            Duration::from_secs(10),
        )?;

        let client =
            CosClient::new(&format!("http://127.0.0.1:{}", vm.port), None)?;

        Ok(Self { vm, client })
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
