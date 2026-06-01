use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));
    println!(
        "cargo::rerun-if-changed={}",
        proto_root.to_str().unwrap()
    );

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .extern_path(
            ".containeros.shared.v1",
            "::cos-api-shared::proto::v1",
        )
        .extern_path(
            ".containeros.reconciler.v1",
            "::cos-api-reconciler::proto::v1",
        )
        .compile_protos(
            &[proto_root.join("containeros/reconciler/v1/service.proto")],
            &[proto_root],
        )?;

    Ok(())
}
