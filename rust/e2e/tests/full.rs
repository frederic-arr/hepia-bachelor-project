#[cfg(test)]
mod validation {

    use std::time::Duration;

    use cosc::Key;
    use e2e::{CosVm, random_port, wait_for_request};
    use serde_json::json;

    async fn create_vm() -> CosVm {
        CosVm::new(Some(env!("CARGO_TARGET_TMPDIR")), None, vec![])
            .await
            .unwrap()
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
        assert_eq!(resources.len(), 8);

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn create_config() {
        let data = include_str!("./data/create-config.yaml");

        let mut vm = create_vm().await;
        let () = vm.push_str(data).await.unwrap();

        std::thread::sleep(Duration::from_secs(5));
        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 7);

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn auth() {
        let data = include_str!("./data/auth.yaml");

        let mut vm = create_vm().await;
        let () = vm.push_str(data).await.unwrap();
        vm.list().await.unwrap_err();

        vm.set_password(Some("hepia2026demo".to_owned()));
        std::thread::sleep(Duration::from_secs(5));

        let resources = vm.list().await.unwrap();
        assert_eq!(resources.len(), 7);

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
    async fn create_rootless_container() {
        let port = random_port();
        let data = include_str!("./data/create-rootless-container.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn networks() {
        let port = random_port();
        let data = include_str!("./data/networks.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        let data = wait_for_request(port).await.unwrap();
        assert!(data.contains("Hello, world!"));

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn volumes() {
        let mut vm = create_vm().await;

        let port1 = random_port();
        let data1 = include_str!("./data/volumes--a.yaml")
            .replace("%%PORT1%%", &port1.to_string());

        let () = vm.push_str(&data1).await.unwrap();
        let data = wait_for_request(port1).await.unwrap();
        assert!(data.contains("AAAAAAAAAAAAAAAA"));

        let () = vm
            .push_str(include_str!("./data/volumes--down.yaml"))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(5)).await;

        let port2 = random_port();
        let data2 = include_str!("./data/volumes--b.yaml")
            .replace("%%PORT2%%", &port2.to_string());

        let () = vm.push_str(&data2).await.unwrap();
        let data = wait_for_request(port2).await.unwrap();
        assert!(data.contains("AAAAAAAAAAAAAAAA"));
        assert!(data.contains("BBBBBBBBBBBBBBBB"));

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn rootless_networks() {
        let port = random_port();
        let data = include_str!("./data/rootless-networks.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        let data = wait_for_request(port).await.unwrap();
        assert!(data.contains("Hello, world!"));

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "TODO"]
    async fn create_3tier() {
        let port1 = random_port();
        let port2 = random_port();
        let port3 = random_port();
        let data = include_str!("./data/create-3tier.yaml")
            .replace("%%PORT1%%", &port1.to_string())
            .replace("%%PORT2%%", &port2.to_string())
            .replace("%%PORT3%%", &port3.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();

        let (a, b, c) = tokio::join!(
            wait_for_request(port1),
            wait_for_request(port2),
            wait_for_request(port3),
        );
        a.unwrap();
        b.unwrap();
        c.unwrap();

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn install() {
        let port = random_port();
        let data = include_str!("./data/install.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = CosVm::new(
            Some(env!("CARGO_TARGET_TMPDIR")),
            Some(1024),
            vec![],
        )
        .await
        .unwrap();
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn reboot() {
        let port = random_port();
        let data = include_str!("./data/install.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = CosVm::new(
            Some(env!("CARGO_TARGET_TMPDIR")),
            Some(1024),
            vec![],
        )
        .await
        .unwrap();
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();

        // TODO: Should have a proper reboot command instead of just killing the
        // VM
        tokio::time::sleep(Duration::from_secs(5)).await;

        vm.reboot().await.unwrap();
        wait_for_request(port).await.unwrap();

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn publish_port() {
        let probe_port = random_port();
        let data = include_str!("./data/publish-port.yaml")
            .replace("%%PORT%%", &probe_port.to_string());

        let mut vm = CosVm::new(
            Some(env!("CARGO_TARGET_TMPDIR")),
            None,
            vec![8080],
        )
        .await
        .unwrap();
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(probe_port).await.unwrap();

        let http_port = vm.get_port(8080).unwrap();
        let body = reqwest::get(format!("http://127.0.0.1:{http_port}"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        assert_eq!(body, "Hello, world!\n");

        vm.kill().await.unwrap();
    }

    #[tokio::test]
    async fn create_delete_container() {
        let port = random_port();
        let data = include_str!("./data/create-delete-container--create.yaml")
            .replace("%%PORT%%", &port.to_string());

        let mut vm = create_vm().await;
        let () = vm.push_str(&data).await.unwrap();
        wait_for_request(port).await.unwrap();

        let data = include_str!("./data/create-delete-container--delete.yaml");
        let () = vm.push_str(data).await.unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        vm.get_resource(&Key {
            schema: "container:instance".to_owned(),
            name: Some("probe".to_owned()),
        })
        .await
        .unwrap_err();

        vm.kill().await.unwrap();
    }
}
