use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(true)
        .compile_protos(
            &[proto_root.join("containeros/system_manager/internal/server/v1/server.proto")],
            &[proto_root],
        )?;

    Ok(())
}
