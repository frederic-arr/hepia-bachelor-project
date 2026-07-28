#[cfg(test)]
mod validation {

    use cosc::Key;
    use e2e::{CosVm, random_port, wait_for_request};
    use serde_json::json;

    async fn create_vm() -> CosVm {
        CosVm::new(Some(env!("CARGO_TARGET_TMPDIR"))).await.unwrap()
    }

    #[tokio::test]
    async fn starts() {
        let _vm = create_vm().await;
    }

    #[tokio::test]
    async fn get_route() {
        let mut vm = create_vm().await;
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
        let mut vm = create_vm().await;
        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 7);

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn create_config() {
        let port = random_port();
        let data = include_str!("./data/create-container.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        vm.set_password(Some("hepia2026demo".to_owned()));

        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 10);

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn create_container() {
        let port = random_port();
        let data = include_str!("./data/create-container.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "TODO"]
    async fn create_delete_container() {
        let port = random_port();
        let data = include_str!("./data/create-delete-container--create.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        vm.set_password(Some("hepia2026demo".to_owned()));
        wait_for_request(port).await.unwrap();

        let data = include_str!("./data/create-delete-container--delete.yaml");
        let () = vm.push_str(data).await.unwrap();

        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 7);

        vm.kill().await.unwrap();
    }
}
