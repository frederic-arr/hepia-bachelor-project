#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'
mod proto 'proto/justfile'

@help:
    just --list

[parallel]
check: docs::check proto::check

[parallel]
fix: docs::fix proto::fix

[parallel]
build: docs::build

[parallel]
clean: docs::clean

[private]
_pre-commit: fix check
