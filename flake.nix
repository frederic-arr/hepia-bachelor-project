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

        crossSystem = "armv6l-unknown-linux-gnueabihf";
        crossPkgs = import nixpkgs {
          localSystem = system;
          crossSystem = crossSystem;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust/rust-toolchain.toml);

        crossRustToolchain = pkgs:
          (pkgs.rust-bin.fromRustupToolchainFile ./rust/rust-toolchain.toml).override {
            targets = [ "x86_64-unknown-linux-musl" "arm-unknown-linux-gnueabihf" ];
          };

        rustFn = pkgs.callPackage ./rust {
          inherit crane rustToolchain;
          inherit (pkgs) lib protobuf;
        };

        rustFn-rpi = crossPkgs.callPackage ./rust {
          inherit crane;
          rustToolchain = crossRustToolchain;
          lib = crossPkgs.lib;
          protobuf = crossPkgs.protobuf;
        };

        kernelFn = pkgs.callPackage ./linux { };
        kernelFn-cross = crossPkgs.callPackage ./linux { };

        rpiKernelSrc = pkgs.fetchFromGitHub {
          owner  = "raspberrypi";
          repo   = "linux";
          rev    = "refs/heads/rpi-6.18.y";
          hash   = "sha256-wOA7rhawFLNsbCMRBzV8bdS4fSrm9KB4SQLDu1cbcD4=";
        };

        rpi1 = kernelFn-cross {
          arch      = "arm";
          base      = "bcmrpi_defconfig";
          fragments = [ ./linux/config/common.conf ];
          src       = rpiKernelSrc;
          version   = "6.18.38";
        };

        x86_64-generic = kernelFn {
          arch      = "x86_64";
          base      = "defconfig";
          fragments = [ ./linux/config/common.conf ];
        };

        e2eTests = pkgs.callPackage ./rust/e2e.nix {
          inherit crane rustToolchain;
          inherit (pkgs) lib protobuf;
        } { inherit iso; };

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
          ];
        };

        rootfsEnv = pkgs.buildEnv {
          name   = "rootfs-env";
          paths  = [ supervisor statemgr netctl conctl sysctl pkgs.podman pkgs.busybox pkgs.cacert pkgs.gptfdisk pkgs.e2fsprogs pkgs.util-linux pkgs.limine ];
          pathsToLink = [ "/bin" "/lib" "/etc" "/share" ];
        };

        # Inspired by https://github.com/NixOS/nixpkgs/blob/26.05/nixos/lib/make-squashfs.nix
        # Which seems to be the source somewhat?
        # https://refspecs.linuxfoundation.org/FHS_3.0/fhs/index.html
        # TODO: Add compression?
        rootfs = pkgs.runCommand "mkrootfs" { } ''
          closureInfo=${pkgs.closureInfo { rootPaths = [ rootfsEnv ]; }}
          mkdir -p source/nix/store
          mkdir -p source/{bin,lib,share}
          mkdir -p source/{dev,proc,sys}
          mkdir -p source/{etc,home,media,mnt,opt,run,sbin,srv,tmp,usr,var}

          cp -a ${rootfsEnv}/. source/

          cp "$closureInfo/registration" source/nix/store/

          # store-paths is a file containing all the paths. In the original script
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

        qemu-boot-x86_64 = pkgs.runCommand "boot-x86_64" { } ''
          mkdir -p $out
          cp ${x86_64-generic.kernel}/bzImage $out/bzImage
          cp ${initrd}/initrd                 $out/initrd
          cp ${rootfs}                        $out/root.squashfs
        '';

        iso = pkgs.runCommand "embedded-x86_64.iso" {
          nativeBuildInputs = with pkgs; [
            grub2
            grub2_efi
            xorriso
            mtools
          ];
        } ''
          mkdir -p iso/boot/grub

          cp ${x86_64-generic.kernel}/bzImage iso/boot/bzImage
          cp ${initrd}/initrd                  iso/boot/initrd
          cp ${rootfs}                         iso/root.squashfs

          cat > iso/boot/grub/grub.cfg << 'EOF'
          set timeout=0

          menuentry "ContainerOS" {
            linux  /boot/bzImage init=/init console=ttyS0,115200 cos.maintenance
            initrd /boot/initrd
          }
          EOF

          grub-mkrescue \
            --modules="linux iso9660 squash4 normal boot configfile" \
            -o $out \
            iso/
        '';

        init = rustFn { package = "init"; deps = [ "crates/linux-utils" ]; };
        supervisor = rustFn { package = "supervisor"; deps = [ "crates/linux-utils" ]; };
        netctl = rustFn { package = "network-controller"; deps = [
          "crates/cos-proto-reconciler"
          "crates/cos-proto-reconciler-server"
          "crates/isolation"
          "crates/isolation-macros"
          "crates/linux-utils"

          "crates/cos-proto-state"
          "crates/cos-proto-state-client"
          "cmd/system-controller"
        ]; };

        conctl = rustFn { package = "container-controller"; deps = [
          "crates/cos-proto-reconciler"
          "crates/cos-proto-reconciler-server"
          "crates/isolation"
          "crates/isolation-macros"
          "crates/linux-utils"

          "crates/cos-proto-state"
          "crates/cos-proto-state-client"
          "cmd/system-controller"
        ]; };

        sysctl = rustFn { package = "system-controller"; deps = [
          "crates/cos-proto-reconciler"
          "crates/cos-proto-reconciler-server"
          "crates/isolation"
          "crates/isolation-macros"
          "crates/linux-utils"
        ]; };

        statemgr = rustFn { package = "state-manager"; deps = [
          "crates/cos-proto-reconciler"
          "crates/cos-proto-reconciler-server"
          "crates/isolation"
          "crates/isolation-macros"
          "crates/linux-utils"

          "crates/cos-proto-state"
          "crates/cos-proto-state-client"

          "crates/cos-proto-reconciler-client"
          "crates/cos-proto-state-server"

          "cmd/network-controller"
          "cmd/container-controller"
          "cmd/system-controller"
        ]; };

        init-rpi = rustFn-rpi { package = "init"; deps = [ "crates/linux-utils" ]; };
        supervisor-rpi = rustFn-rpi { package = "supervisor"; deps = [ "crates/linux-utils" ]; };
        netctl-rpi = rustFn-rpi { package = "network-controller"; deps = [ "crates/linux-utils" "crates/cos-proto-reconciler" "crates/cos-proto-reconciler-server" "cmd/system-controller" ]; };
        sysctl-rpi = rustFn-rpi { package = "system-controller"; deps = [ "crates/cos-proto-reconciler" "crates/cos-proto-reconciler-server" ]; };
        statemgr-rpi = rustFn-rpi { package = "state-manager"; deps = [ "crates/cos-proto-reconciler" "crates/cos-proto-reconciler-client" "crates/cos-proto-reconciler-server" "cmd/network-controller" "cmd/system-controller" ]; };

        rpi1-rootfsEnv = crossPkgs.buildEnv {
          name = "rootfs-env-rpi1";
          paths = [ supervisor statemgr netctl sysctl pkgs.podman pkgs.busybox pkgs.cacert pkgs.util-linux ];
          pathsToLink = [ "/bin" "/lib" "/etc" "/share" ];
        };

        rpi1-rootfs = pkgs.runCommand "mkrootfs-rpi1" { } ''
          closureInfo=${crossPkgs.closureInfo { rootPaths = [ rpi1-rootfsEnv ]; }}
          mkdir -p source/nix/store
          mkdir -p source/{bin,lib,share}
          mkdir -p source/{dev,proc,sys}
          mkdir -p source/{etc,home,media,mnt,opt,run,sbin,srv,tmp,usr,var}

          cp -a ${rpi1-rootfsEnv}/. source/

          cp "$closureInfo/registration" source/nix/store/

          # store-paths is a file containing all the paths. In the original script
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

        rpi1-initrd = crossPkgs.makeInitrdNG {
          contents = [
            { target = "/init";    source = "${init-rpi}/bin/init"; }
            { target = "/busybox"; source = "${crossPkgs.busybox}/bin/busybox"; }
          ];
        };

        rpiFirmware = pkgs.fetchFromGitHub {
          owner = "raspberrypi";
          repo = "firmware";
          rev = "1.20260521";
          hash = "sha256-zoxAq2VewNqexO0MTknLdi/u3zVYGsS0mqlLyaAtJp8=";
        };

        # TODO: This doesn't work. Kernel cannot find the init?
        rpi1-sd-image = pkgs.runCommand "rpi1-sd-image.img" {
          nativeBuildInputs = with pkgs; [ dosfstools mtools parted ];
        } ''
          diskSize=$(( 128 * 1024 * 1024 ))
          start=$(( 1 * 1024 * 1024 ))
          partSize=$(( diskSize - start ))
          sectorSize=512
          startSector=$(( start / sectorSize ))
          partSectors=$(( partSize / sectorSize ))

          dd if=/dev/zero of=$out bs=$diskSize count=1 status=none
          parted -s $out mklabel msdos
          parted -s $out mkpart primary fat32 ''${start}B 100%   # use whole remaining space
          parted -s $out set 1 boot on

          mkfs.vfat -C boot.img $partSectors

          mcopy -i boot.img -s ${rpiFirmware}/boot/* ::/

          mcopy -i boot.img -o ${rpi1.kernel}/zImage ::/kernel.img
          mcopy -i boot.img ${rpi1-initrd}/initrd ::/initramfs
          mcopy -i boot.img ${rpi1-rootfs}        ::/root.squashfs

          cat > config.txt <<EOF
          kernel=kernel.img
          initramfs initramfs followkernel
          arm_boost=0
          EOF
          # mcopy -i boot.img config.txt ::/config.txt

          echo "console=serial0,115200 console=tty1 rootwait quiet init=/init splash cos.maintenance" > cmdline.txt
          mcopy -i boot.img cmdline.txt ::/cmdline.txt

          dd if=boot.img of=$out bs=$sectorSize seek=$startSector conv=notrunc status=none
        '';

        cspellDictFr = pkgs.stdenvNoCC.mkDerivation {
          name = "cspell-dict-fr-fr";
          src = pkgs.fetchurl {
            url = "https://registry.npmjs.org/@cspell/dict-fr-fr/-/dict-fr-fr-2.3.2.tgz";
            hash = "sha256-zOsyxv7XQBucK6m93JaOrT56qo5m0IHRq+qMrDVXXEw=";
          };
          dontBuild = true;
          dontConfigure = true;
          installPhase = ''
            mkdir -p $out
            cp -r . $out/
          '';
        };
      in
      {
        formatter = pkgs.nixfmt-tree;
        packages = {
          inherit qemu-boot-x86_64 iso rootfs initrd init supervisor netctl rpi1-sd-image;
          kernel-x86_64-generic = x86_64-generic.kernel;
        };
        apps = {
          menuconfig-x86_64-generic = {
            type = "app";
            program = "${x86_64-generic.menuconfig}/bin/menuconfig";
          };
        };
        checks.e2e = e2eTests;
        devShells = {
          default = pkgs.mkShellNoCC {
            packages = with pkgs; [
              pkg-config stdenv.cc ncurses gnumake flex bison just just-lsp
              jq pre-commit protobuf typst typstyle tinymist buf plantuml
              rustToolchain llvmPackages.libclang clang linuxHeaders bison flex
              perl bc openssl rsync gmp libmpc mpfr elfutils zstd python3Minimal
              kmod hexdump cargo-nextest cspell
            ];
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            CPATH = "${pkgs.linuxHeaders}/include";
            BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.linuxHeaders}/include";
            NIX_CFLAGS_COMPILE = "-I${pkgs.linuxHeaders}/include";
            shellHook = ''
              cat > .config/.cspell.json <<EOF
              {
                "import": [
                  "${cspellDictFr}/cspell-ext.json",
                  "./cspell.yaml"
                ]
              }
              EOF
            '';
          };
        };
      }
    );
}
