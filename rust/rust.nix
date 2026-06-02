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
    ] ++ map (p: craneLib.fileset.commonCargoSources ./crates/${p}) deps);
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    # Will be overriden later but set to supress warning during buildDepsOnly
    nativeBuildInputs = [ protobuf ];
    preBuild = ''
      mkdir -p $sourceRoot/cmd
      mkdir -p $sourceRoot/crates
      cp -r ${protoSrc} ./proto
    '';
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
    pname = "workspace";
    version = "0.1.0";
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
