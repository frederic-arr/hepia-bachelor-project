{ lib, stdenv, pkgs, linuxKernel, fetchurl, runCommand }:
{ arch, base, fragments, patches ? [] }:

let
  version = "6.19.9";
  src = fetchurl {
    url  = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${version}.tar.xz";
    hash = "sha256-wWBoo68S45Q97jse71fKcCKcBpEov6EYT7P0iyGdVb8=";
  };

  storeFragments = map (f: builtins.path { path = f; name = builtins.baseNameOf f; }) fragments;
  storePatches = map (f: builtins.path { path = f; name = builtins.baseNameOf f; }) patches;

  mergedConfig = runCommand "kernel-config-${arch}-${base}" {
    nativeBuildInputs = with pkgs; [
      stdenv.cc
      flex
      bison
    ];
  } ''
    tar -xf ${src} --strip-components=1
    for p in ${toString storePatches}; do
      patch -p1 < "$p"
    done
    make ARCH=${arch} ${base}
    scripts/kconfig/merge_config.sh -m .config ${toString storeFragments}
    cp .config $out
  '';

in
linuxKernel.manualConfig {
  inherit lib stdenv version src;
  modDirVersion = version;
  configfile    = mergedConfig;
  allowImportFromDerivation = true;
  kernelPatches = map (p: { patch = p; }) storePatches; # I hope I never have to patch the kernel
}
