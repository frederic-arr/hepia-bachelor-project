# Source: https://www.thenegation.com/posts/nixos-on-qemu/
# ``sh
# nix build -L --impure --expr '(import <nixpkgs/nixos> { configuration = ./vm.nix; }).vm'
# export QEMU_KERNEL_PARAMS="console=ttyS0"
# export QEMU_NET_OPTS="hostfwd=tcp:127.0.0.1:2222-:22"
# ./result/bin/run-nixos-vm -nographic; reset
# ```

{ pkgs, ... }:
{
  imports = [
    <nixpkgs/nixos/modules/virtualisation/qemu-vm.nix>
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

  services.openssh.enable = true;

  virtualisation.memorySize = 180;
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
