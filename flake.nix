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
        kernelFn = pkgs.callPackage ./kernel/kernel.nix { };
        rustFn = pkgs.callPackage ./rust/rust.nix {
          inherit crane rustToolchain;
          inherit (pkgs) lib protobuf;
        };

        kernel-x86_64-generic = kernelFn {
          arch      = "x86_64";
          base      = "defconfig";
          fragments = [ ];
        };

        initramfs = pkgs.makeInitrdNG {
          contents = [
            {
              target = "/init";
              source  = "${initd}/bin/init";
            }
          ];
        };

        initd = rustFn {
          package = "init";
          deps = [ "invariant-macros" ];
        };
      in
      {
        formatter = pkgs.nixfmt-tree;
        packages = {
          inherit kernel-x86_64-generic;
          inherit initramfs;
          inherit initd;

          netmgr = rustFn {
            package = "network-manager";
            deps = [ "invariant-macros" "cos-api-reconciler" "cos-api-reconciler-server" ];
          };

          qemu-boot-x86_64 = pkgs.runCommand "boot-x86_64" { } ''
            mkdir -p $out
            cp ${kernel-x86_64-generic}/bzImage $out/bzImage
            cp ${initramfs}/initrd.gz           $out/initramfs.gz
          '';
        };

        devShells.default = pkgs.mkShellNoCC {
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
          ];
        };
      }
    );
}
