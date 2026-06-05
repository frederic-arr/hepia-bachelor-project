# 2026-06-03

- overlayfs cannot be nested because *something something* "whiteout"
- It was DNS


mkdir /etc/containers
echo '{"default":[{"type":"insecureAcceptAnything"}]}' > /etc/containers/policy.json
echo 'nameserver 9.9.9.9' > /etc/resolv.conf
ip link set dev eth0 up
ip addr add 10.0.2.15/24 dev eth0
ip route add default via 10.0.2.2
podman --log-level=trace pull --tls-verify=false docker.io/library/busybox:latest
podman --log-level=trace run --rm -it docker.io/library/busybox:latest



podman --log-level=trace run --rm -it --network none --cgroups disabled docker.io/library/busybox:latest
podman --log-level=trace run --rm -it --network none docker.io/library/busybox:latest


nix run -L .#menuconfig-x86_64-generic
nix build -L .#qemu-boot-x86_64
qemu-system-x86_64 \
  -kernel result/bzImage -initrd result/initrd \
  -enable-kvm \
  -cpu host -m 720M \
  -netdev user,id=net0,hostfwd=tcp::1234-:1234 \
  -device e1000,netdev=net0 \
  -nographic -append "console=ttyS0"
