# TODO

- [ ] **DUE: 15/05/26** Write first draft of Subject statement
- [ ] **DUE: 22/05/26** Plan work
- [ ] Document usage of AI
- [ ] Prepare thesis structure
- [ ] Setup dev env

**Run the thing**:
```sh
nix build -L .#iso 
qemu-system-x86_64 -cdrom result -drive file=disk.img,format=raw,if=virtio \
  -enable-kvm \
  -cpu host -m 2560M \
  -netdev user,id=net0,hostfwd=tcp::50000-:50000 \
  -device e1000,netdev=net0 \
  -nographic
cargo run -p cosc -- --server http://127.0.0.1:50000 push --config ../data/config.yaml
```

**Run E2E Tests**:
```sh
nix build -L .#checks.x86_64-linux.e2e
```
