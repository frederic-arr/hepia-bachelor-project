{ pkgs, lib, stdenv, linuxKernel, fetchurl, runCommand }:
{ arch, base, fragments ? [ ], patches ? [ ] }:

let
  version = "6.19.9";
  src = fetchurl {
    url = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-${version}.tar.xz";
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

  kernel = linuxKernel.manualConfig {
    inherit lib stdenv version src;
    modDirVersion = version;
    configfile    = mergedConfig;
    allowImportFromDerivation = true;
    kernelPatches = map (p: { patch = p; }) storePatches; # I hope I never have to patch the kernel
  };

  menuconfig = pkgs.writeShellApplication {
    name = "menuconfig";

    runtimeInputs = with pkgs; [
      bc
      bison
      flex
      gnumake
      ncurses
      ncurses.dev
      pkg-config
      diffutils
      stdenv.cc
    ];

    text = ''
      set -euo pipefail

      export HOSTCC="${pkgs.stdenv.cc}/bin/cc"
      export HOSTCFLAGS="-I${pkgs.ncurses.dev}/include"
      export HOSTLDFLAGS="-L${pkgs.ncurses.out}/lib"
      export NIX_LDFLAGS="-L${pkgs.ncurses.out}/lib"

      workdir="$PWD/kernel"
      outfull="$PWD/kernel/config.full"
      outmerged="$PWD/kernel/config.merged"
      outdiff="$PWD/kernel/config.diff"
      cd "$workdir"

      if [ ! -d linux-${version} ]; then
        tar -xf ${src}
      fi

      cd linux-${version}
      export ARCH=${arch}
      export KCONFIG_CONFIG="$PWD/.config"
      rm -f .config .config.baseline .config.current

      make ${base}
      cp .config .config.baseline

      ${lib.optionalString (fragments != [ ]) ''
        scripts/kconfig/merge_config.sh -m .config ${builtins.toString fragments}
      ''}

      cp .config .config.current

      make menuconfig

      scripts/diffconfig -m .config.current .config > "$outdiff"
      scripts/diffconfig -m .config.baseline .config > "$outmerged"
      cp .config "$outfull"

      echo "Menuconfig changes: $outdiff"
      echo "Diff from ${base}: $outmerged"
      echo "Full configuration: $outfull"
    '';
  };
in
{
  inherit kernel menuconfig src version mergedConfig;
}
