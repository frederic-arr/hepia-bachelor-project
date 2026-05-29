use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(false)
        .extern_path(
            ".containeros.shared.v1",
            "::cos-api-shared::proto::v1",
        )
        .compile_protos(
            &[proto_root.join("containeros/reconciler/v1/service.proto")],
            &[proto_root],
        )?;

    Ok(())
}
