#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'
mod proto 'proto/justfile'
mod rust 'scripts/justfile'
mod kernel 'kernel/justfile'

@help:
    just --list

[parallel]
check: docs::check proto::check rust::check

[parallel]
fix: docs::fix proto::fix rust::fix

[parallel]
build: docs::build proto::build rust::build

[parallel]
clean: docs::clean proto::clean rust::clean

[private]
[env("RUSTFLAGS", "-D warnings")]
_pre-commit: check

[working-directory: 'kernel/linux-6.19.9']
quick:
    rm .config
    make ARCH=x86_64 defconfig
    ./scripts/kconfig/merge_config.sh -m .config ../shared/common.conf
    make ARCH=x86_64 olddefconfig
    make -j 16 ARCH=x86_64 bzImage
    qemu-system-x86_64 \
        -kernel arch/x86/boot/bzImage -initrd ../../result/initrd \
        -enable-kvm \
        -cpu host -m 720M \
        -netdev user,id=net0,hostfwd=tcp::1234-:1234 \
        -device e1000,netdev=net0 \
        -nographic -append "console=ttyS0"
