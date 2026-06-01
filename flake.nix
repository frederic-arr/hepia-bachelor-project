{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
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
        kernelFn = pkgs.callPackage ./kernel/kernel.nix { };
      in
      {
        formatter = pkgs.nixfmt-tree;
        packages = {
          kernel-x86_64-generic = kernelFn {
            arch      = "x86_64";
            base      = "defconfig";
            fragments = [ ];
          };
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
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          ];
        };
      }
    );
}
