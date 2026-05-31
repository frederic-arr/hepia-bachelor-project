#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'
mod proto 'proto/justfile'
mod rust 'scripts/justfile'

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
