# Source: https://www.thenegation.com/posts/nixos-on-qemu/
# ## ISO
# ```sh
# nix build -L --impure --expr '(import <nixpkgs/nixos> { configuration = ./vm.nix; }).config.system.build.isoImage'
# qemu-system-x86_64 -cdrom result/iso/* -drive file=disk.img,format=raw,if=virtio \
#   -enable-kvm -smp 4 \
#   -cpu host -m 276M \
#   -netdev user,id=net0,hostfwd=tcp::2222-:22 \
#   -device e1000,netdev=net0 \
#   -nographic
# ```
#
# ## Direct boot
# ``sh
# nix build -L --impure --expr '(import <nixpkgs/nixos> { configuration = ./vm.nix; }).vm'
# export QEMU_KERNEL_PARAMS="console=ttyS0"
# export QEMU_NET_OPTS="hostfwd=tcp:127.0.0.1:2222-:22"
# ./result/bin/run-nixos-vm -nographic; reset
# ```

{ pkgs, ... }:
{
  imports = [
    # <nixpkgs/nixos/modules/virtualisation/qemu-vm.nix>
    <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix>
  ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking.firewall.allowedTCPPorts = [ 22 ];

  users.users.root = {
    initialPassword = "hepia";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBb+4f8zS5R7ErqHKKNP70wt3YLWYuNCZtdsbUvm48S"
    ];
  };

  systemd.services.autoinstall = {
    description = "Automatic NixOS install";
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      ExecStart = "/etc/install.sh";
      RemainAfterExit = true;
    };
  };

  environment.etc."install.sh" = {
    mode = "0755";
    text = ''
      #!/bin/sh
      set -eux

      # Partition disk
      parted /dev/vda -- mklabel gpt
      parted /dev/vda -- mkpart ESP fat32 1MiB 512MiB
      parted /dev/vda -- set 1 esp on
      parted /dev/vda -- mkpart primary 512MiB 100%

      mkfs.fat -F32 /dev/vda1
      mkfs.ext4 -F /dev/vda2

      mount /dev/vda2 /mnt
      mkdir -p /mnt/boot
      mount /dev/vda1 /mnt/boot

      nixos-generate-config --root /mnt

      cp /etc/nixos/configuration.nix \
        /mnt/etc/nixos/configuration.nix

      nixos-install --no-root-password --root /mnt

      reboot
    '';
  };

  services.openssh.enable = true;

  # virtualisation.memorySize = 180;
  virtualisation.containers.enable = true;
  virtualisation.podman.enable = true;
  virtualisation.oci-containers.backend = "podman";
  virtualisation.oci-containers.containers = {
    probe = {
      image = "docker.io/alpine/curl:latest";
      autoStart = true;
      cmd = [ "http://10.0.2.2:1234" ];
    };
  };

  system.stateVersion = "26.05";
}
