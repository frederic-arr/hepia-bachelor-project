#!/usr/bin/env -S just --justfile

mod docs 'docs/internal/justfile'

@help:
  just --list

[doc('Performs linting')]
[parallel]
lint: docs::lint

[doc('Performs linting')]
[parallel]
compile: docs::compile
