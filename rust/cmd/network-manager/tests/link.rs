use cos_api_reconciler::{Identity, ReconcileDynamicResourceRequest};
use cos_api_reconciler_server::Reconcilable;
use network_manager::{Link, LinkSpec, LinkState, LinkType};
use rtnetlink::{LinkDummy, new_connection};

#[test]
#[isolation::isolate]
fn test_full() {
    let (conn, mut ctx, _) = new_connection().unwrap();
    tokio::spawn(conn);

    let request = ReconcileDynamicResourceRequest::<LinkSpec, LinkState> {
        schema: Link::SCHEMA.to_string(),
        name: "dummy0".to_string(),
        spec: LinkSpec {
            link_type: LinkType::Dummy,
            admin_up: true,
            mtu: None,
            address: None,
            broadcast: None,
            altnames: vec![],
            arp: true,
            promiscuous: false,
        },
        state: None,
        children: vec![],
        owner: Identity {
            schema: "".to_string(),
            name: "".to_string(),
        },
    };

    let output = Link::reconcile(&mut ctx, &request).await.unwrap();
    assert!(output.state.len() > 0);
}
