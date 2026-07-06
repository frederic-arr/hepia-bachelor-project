use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));
    println!(
        "cargo::rerun-if-changed={}",
        proto_root.to_str().unwrap()
    );

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(true)
        .extern_path(".containeros.api.v1", "::cos-api-api::proto::v1")
        .compile_protos(
            &[proto_root.join("containeros/api/v1/service.proto")],
            &[proto_root],
        )?;

    Ok(())
}
