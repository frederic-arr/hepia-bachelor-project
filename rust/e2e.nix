{ crane, pkgs, lib, rustToolchain, protobuf }:
{ iso }:

let
  deps = [ "invariant-macros" ];
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
      (craneLib.fileset.commonCargoSources ./cmd/e2e-tests)
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
    cargoToml = ./cmd/e2e-tests/Cargo.toml;
  }) version;
  src = crateSrc;
  pname = "e2e-tests";

  cargoExtraArgs = "--offline -p e2e-tests";
  E2E_DISK_IMAGE = iso;
})
