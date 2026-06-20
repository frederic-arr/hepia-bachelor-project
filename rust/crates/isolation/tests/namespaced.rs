use isolation::namespaced;
use rtnetlink::LinkDummy;

#[test]
fn test_fs() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || async {
        let r = std::fs::read_dir("/").unwrap();
        for f in r {
            dbg!(f);
        }
    });
}

#[test]
fn test_full() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || async {
        use rtnetlink::new_connection;

        let (conn, handle, _) = new_connection().unwrap();
        tokio::spawn(conn);
        handle
            .link()
            .add(LinkDummy::new("dummy0").build())
            .execute()
            .await
            .unwrap();
    });
}

#[test]
#[expect(clippy::assertions_on_constants)]
fn test_a() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || async {
        assert!(true);
    });
}

#[test]
#[should_panic = "explicit panic"]
#[expect(clippy::assertions_on_constants)]
fn test_b() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || async {
        assert!(false);
    });
}
