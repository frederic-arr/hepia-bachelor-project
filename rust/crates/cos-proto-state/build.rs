#![allow(clippy::unwrap_used, reason = "this is a build script")]

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from(env!("PROTO_ROOT"));
    println!(
        "cargo::rerun-if-changed={}",
        proto_root.to_str().unwrap()
    );

    tonic_prost_build::configure()
        .build_client(false)
        .build_server(false)
        .compile_protos(
            &[proto_root.join("containeros/state/v1/service.proto")],
            &[proto_root],
        )?;

    Ok(())
}
