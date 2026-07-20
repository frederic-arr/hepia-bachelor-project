#![expect(clippy::tests_outside_test_module, reason = "https://github.com/rust-lang/rust-clippy/issues/11024")]

use isolation::namespaced;

#[test]
#[expect(clippy::assertions_on_constants, reason = "we want to force a panic")]
fn normal_test_case_should_work() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || assert!(true));
}

#[test]
#[should_panic = "assertion failed: false"]
#[expect(clippy::assertions_on_constants, reason = "we want to force a panic")]
fn panics_should_work() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || assert!(false));
}

#[test]
fn filesystem_should_be_empty() {
    namespaced(env!("CARGO_TARGET_TMPDIR"), || {
        let r = std::fs::read_dir("/").unwrap();
        assert_eq!(r.count(), 1);
    });
}

#[test]
fn network_should_be_managable() {
    use rtnetlink::sys::SmolSocket;
    use rtnetlink::{LinkDummy, new_connection_with_socket};

    namespaced(env!("CARGO_TARGET_TMPDIR"), || {
        let (conn, handle, _) =
            new_connection_with_socket::<SmolSocket>().unwrap();
        smol::spawn(conn).detach();
        smol::block_on(
            handle
                .link()
                .add(LinkDummy::new("dummy0").build())
                .execute(),
        )
        .unwrap();
    });
}
