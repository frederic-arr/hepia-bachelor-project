{ crane, pkgs, lib, rustToolchain, protobuf }:
{ iso }:

let
  deps = [
    "cos-proto-reconciler"
    "cos-proto-api"
    "cos-proto-api-client"
  ];
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  src = craneLib.cleanCargoSource ./.;
  protoSrc = pkgs.lib.cleanSource ./../proto;

  crateSrc = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions ([
      ./.cargo
      ./.config
      ./Cargo.toml
      ./Cargo.lock
      ./e2e # don't filter because we might have non-rust stuff that's still important such as fixtures
      ./cmd/cosc
    ] ++ map (p: craneLib.fileset.commonCargoSources ./crates/${p}) deps);
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    # Will be overriden later but set to supress warning during buildDepsOnly
    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.pkgsBuildBuild.rustPlatform.bindgenHook
      protobuf
      pkgs.qemu
      pkgs.e2fsprogs
    ];

    postUnpack = ''
      cp -r ${protoSrc} $sourceRoot/../proto
      chmod -R u+w $sourceRoot/../proto
    '';
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
    pname = "e2e-workspace";
    version = "0.1.0";
  });
in
craneLib.cargoNextest (commonArgs // {
  inherit cargoArtifacts;
  inherit (craneLib.crateNameFromCargoToml {
    cargoToml = ./e2e/Cargo.toml;
  }) version;
  src = crateSrc;
  pname = "e2e";

  cargoExtraArgs = "--offline -p e2e";
  E2E_DISK_IMAGE = iso;

  postInstall = ''
    mkdir -p $out
    cp target/nextest/default/junit.xml $out/junit.xml || true
  '';
})
