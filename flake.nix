{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust/rust-toolchain.toml);
        kernelFn = pkgs.callPackage ./kernel { };
        rustFn = pkgs.callPackage ./rust/rust.nix {
          inherit crane rustToolchain;
          inherit (pkgs) lib protobuf;
        };

        x86_64-generic = kernelFn {
          arch      = "x86_64";
          base      = "defconfig";
          fragments = [ ./kernel/common/common.conf ];
        };

        initrd = pkgs.makeInitrdNG {
          contents = [
            {
              target = "/init";
              source  = "${init}/bin/init";
            }
            {
              target = "/busybox";
              source  = "${pkgs.busybox}/bin/busybox";
            }
            {
              target = "/root.squashfs";
              source  = rootfs;
            }
          ];
        };

        rootfsEnv = pkgs.buildEnv {
          name   = "rootfs-env";
          # paths  = [ supervisor netmgr conmgr sysmgr pkgs.podman pkgs.busybox ];
          paths  = [ supervisor pkgs.podman pkgs.busybox ];
          pathsToLink = [ "/bin" "/lib" ];
        };

        # Inspired by https://github.com/NixOS/nixpkgs/blob/26.05/nixos/lib/make-squashfs.nix
        # Which seems to be the source somewhat?
        # https://refspecs.linuxfoundation.org/FHS_3.0/fhs/index.html
        # TODO: Add compression?
        rootfs = pkgs.runCommand "mkrootfs" { } ''
          closureInfo=${pkgs.closureInfo { rootPaths = [ rootfsEnv ]; }}
          mkdir -p source/nix/store
          mkdir -p source/{bin,lib}
          mkdir -p source/{dev,proc,sys}
          mkdir -p source/{etc,home,media,mnt,opt,run,sbin,srv,tmp,usr,var}

          cp -a ${rootfsEnv}/. source/

          cp "$closureInfo/registration" source/nix/store/

          # store-paths is a file containting all the paths. In the original script
          # they `cat` it while calling mksquashfs which uh... "destructures"
          # the filepath and gives them to squash, but since we don't want them
          # directly at the root, that's how we'll do
          while IFS= read -r storePath; do
            cp -a "$storePath" source/nix/store/
          done < "$closureInfo/store-paths"


          SOURCE_DATE_EPOCH=0 ${pkgs.squashfsTools}/bin/mksquashfs source $out \
            -no-hardlinks \
            -all-root \
            -b 1048576 \
            -root-mode 0755
        '';

        init = rustFn {
          package = "init";
          deps = [ "linux-utils" "invariant-macros" ];
        };

        supervisor = rustFn {
          package = "supervisor";
          deps = [ "linux-utils" "invariant-macros" ];
        };

        netmgr = rustFn {
          package = "network-manager";
          deps = [ "invariant-macros" "cos-api-reconciler" "cos-api-reconciler-server" ];
        };

        conmgr = rustFn {
          package = "container-manager";
          deps = [ "invariant-macros" "cos-api-reconciler" "cos-api-reconciler-server" ];
        };

        sysmgr = rustFn {
          package = "system-manager";
          deps = [ "invariant-macros" "cos-api-reconciler" "cos-api-reconciler-client" ];
        };

        qemu-boot-x86_64 = pkgs.runCommand "boot-x86_64" { } ''
          mkdir -p $out
          cp ${x86_64-generic.kernel}/bzImage $out/bzImage
          cp ${initrd}/initrd                 $out/initrd
        '';

        src = pkgs.fetchurl {
          url  = "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.19.9.tar.xz";
          hash = "sha256-wWBoo68S45Q97jse71fKcCKcBpEov6EYT7P0iyGdVb8=";
        };
      in
      {
        formatter = pkgs.nixfmt-tree;
        packages = {
          inherit qemu-boot-x86_64;
          inherit rootfs;
          inherit initrd;
          inherit init;
          inherit supervisor;
          inherit netmgr;

          kernel-x86_64-generic = x86_64-generic.kernel;
        };

        apps = {
          menuconfig-x86_64-generic = {
            type = "app";
            program = "${x86_64-generic.menuconfig}/bin/menuconfig";
          };
        };

        devShells = {
          default = pkgs.mkShellNoCC {
            packages = with pkgs; [
              pkg-config
              stdenv.cc
              ncurses
              gnumake
              flex
              bison
              just
              just-lsp
              jq
              pre-commit
              protobuf
              typst
              typstyle
              tinymist
              buf
              plantuml
              rustToolchain
              llvmPackages.libclang
              clang
              linuxHeaders
            ];

            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            CPATH = "${pkgs.linuxHeaders}/include";
            BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.linuxHeaders}/include";
            NIX_CFLAGS_COMPILE = "-I${pkgs.linuxHeaders}/include";
          };
        };
      }
    );
}
