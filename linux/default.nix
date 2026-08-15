{ pkgs, lib, stdenv, linuxKernel, fetchurl, runCommand }:
{ arch, base
, fragments ? [ ]
, patches ? [ ]
, src ? fetchurl {
    url = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.19.9.tar.xz";
    hash = "sha256-wWBoo68S45Q97jse71fKcCKcBpEov6EYT7P0iyGdVb8=";
  }
, version ? "6.19.9"
}:

let
  storeFragments = map (f: builtins.path { path = f; name = builtins.baseNameOf f; }) fragments;
  storePatches = map (f: builtins.path { path = f; name = builtins.baseNameOf f; }) patches;

  buildPkgs = pkgs.buildPackages;

  prepareSrc = ''
    if [ -d ${src} ]; then
      cp -r ${src}/. .
    else
      tar -xf ${src} --strip-components=1
    fi
  '';

  mergedConfig = runCommand "kernel-config-${arch}-${base}" {
    nativeBuildInputs = with buildPkgs; [ stdenv.cc flex bison ];
  } ''
    export PATH="${buildPkgs.stdenv.cc}/bin:$PATH"
    export HOSTCC="${buildPkgs.stdenv.cc}/bin/gcc"

    ${prepareSrc}
    chmod -R u+w .

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
  };

  menuconfig = pkgs.writeShellApplication {
    name = "menuconfig";
    runtimeInputs = with buildPkgs; [
      bc bison flex gnumake ncurses ncurses.dev pkg-config diffutils stdenv.cc
    ];
    text = ''
      export HOSTCC="${pkgs.stdenv.cc}/bin/cc"
      export HOSTCFLAGS="-I${pkgs.ncurses.dev}/include"
      export HOSTLDFLAGS="-L${pkgs.ncurses.out}/lib"

      workdir="$PWD/linux"
      outfull="$workdir/config.full"
      outmerged="$workdir/config.merged"
      outdiff="$workdir/config.diff"
      cd "$workdir"

      if [ ! -d "linux-${version}" ]; then
        if [ -d "${src}" ]; then
          rm -rf "linux-${version}"
          mkdir "linux-${version}"
          cp -r "${src}/." "linux-${version}"
          chmod -R u+rwX "linux-${version}"
        else
          tar -xf "${src}"
        fi
      fi

      cd linux-${version}
      export ARCH=${arch}
      export KCONFIG_CONFIG="$PWD/.config"
      rm -f .config .config.baseline .config.current

      make ${base}
      cp .config .config.baseline

      ${lib.optionalString (fragments != [ ]) ''
        KCONFIG_CONFIG=.config.custom scripts/kconfig/merge_config.sh -m ${builtins.toString fragments}
      ''}

      ${lib.optionalString (fragments != [ ]) ''
        scripts/kconfig/merge_config.sh -m .config ${builtins.toString fragments}
      ''}

      make olddefconfig
      cp .config .config.before
      make menuconfig
      cp .config .config.after

      scripts/diffconfig -m .config.before .config.after > "$outdiff"
      scripts/diffconfig -m .config.baseline .config.after > "$outmerged"
      cp .config.after "$outfull"

      echo "Menuconfig changes: $outdiff"
      echo "Diff from ${base}: $outmerged"
      echo "Full configuration: $outfull"
    '';
  };
in
{
  inherit kernel menuconfig src version mergedConfig;
}
