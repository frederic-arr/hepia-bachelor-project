#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'

@help:
    just --list

format: docs::format

lint: docs::lint

compile: docs::compile

[private]
_pre-commit: format lint
