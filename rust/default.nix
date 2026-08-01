{ crane, pkgs, lib, rustToolchain, protobuf }:
{ package, deps }:

let
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  src = craneLib.cleanCargoSource ./.;
  protoSrc = pkgs.lib.cleanSource ./../proto;

  crateSrc = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions ([
      ./.cargo
      ./Cargo.toml
      ./Cargo.lock
      (craneLib.fileset.commonCargoSources ./cmd/${package})
    ] ++ map (p: craneLib.fileset.commonCargoSources ./${p}) deps);
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    # Will be overriden later but set to supress warning during buildDepsOnly
    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.pkgsBuildBuild.rustPlatform.bindgenHook
      protobuf
    ];

    postUnpack = ''
      cp -r ${protoSrc} $sourceRoot/../proto
      chmod -R u+w $sourceRoot/../proto
      mkdir -p $sourceRoot/e2e/src
      touch $sourceRoot/e2e/src/lib.rs
      cat <<EOF > $sourceRoot/e2e/Cargo.toml
      [package]
      name = "e2e"
      version = "0.1.0"
      edition = "2024"
      EOF
    '';

    CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
    pname = "workspace";
    version = "0.1.0";
    cargoExtraArgs = "--offline";
  });
in
craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
  inherit (craneLib.crateNameFromCargoToml {
    cargoToml = ./cmd/${package}/Cargo.toml;
  }) version;
  src = crateSrc;
  pname = package;
  doCheck = false;

  cargoExtraArgs = "--offline -p ${package}";
})
