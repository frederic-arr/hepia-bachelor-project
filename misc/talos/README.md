```sh
$ talosctl gen config test https://127.0.0.1:6443 --force \
    --output-types controlplane,talosconfig \
    --with-docs=false \
    --with-cluster-discovery=false \
    --with-examples=false \
    --with-kubespan=false \
    --install-disk /dev/vda \
    --kubernetes-version v1.33.0 \
    --install-image ghcr.io/siderolabs/installer:v1.11.5 \
    --talos-version v1.11.5
$ talosctl machineconfig patch controlplane.yaml --patch @patch.yaml --output config.yaml
$ wget https://github.com/siderolabs/talos/releases/download/v1.11.5/metal-amd64.iso
$ qemu-system-x86_64 -cdrom metal-amd64.iso  -drive file=disk.img,format=raw,if=virtio \
    -enable-kvm \
    -cpu host -m 1024M \
    -netdev user,id=net0,hostfwd=tcp::50000-:50000 \
    -device e1000,netdev=net0 \
    -serial stdio
$ talosctl apply-config --insecure --nodes 127.0.0.1:50000 --file ./config.yaml
$ talosctl bootstrap -e 127.0.0.1:50000 -n 10.0.2.15 --talosconfig=./talosconfig
```

talosctl apply-config--nodes 10.0.2.15 --talosconfig=./talosconfig --file ./config.yaml
