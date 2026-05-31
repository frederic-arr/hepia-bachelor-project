#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'
mod proto 'proto/justfile'
mod scripts 'scripts/justfile'

@help:
    just --list

[parallel]
check: docs::check proto::check scripts::check

[parallel]
fix: docs::fix proto::fix scripts::fix

[parallel]
build: docs::build proto::build scripts::build

[parallel]
clean: docs::clean proto::clean scripts::clean

[private]
_pre-commit: fix check
