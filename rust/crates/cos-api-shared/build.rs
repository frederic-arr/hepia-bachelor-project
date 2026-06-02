use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(false)
        .compile_protos(
            &[proto_root.join("containeros/shared/v1/shared.proto")],
            &[proto_root],
        )?;

    Ok(())
}
