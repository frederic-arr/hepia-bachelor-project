## Reducing memory usage
In crate's `Cargo.toml`:
```toml
cargo-features = ["panic-immediate-abort"]

[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
panic = "immediate-abort"
```

In global `.cargo/config.toml`
```toml
[unstable]
build-std = ["std", "panic_abort"]
build-std-features = ["optimize_for_size"]

[build]
target = "x86_64-unknown-linux-musl"

[target.x86_64-unknown-linux-musl]
rustflags = [
    "-Ctarget-feature=+crt-static",
    "-Zunstable-options",
    "-Cpanic=abort",
    # These options do not shave *as much* in our code yet
    "-Cpanic=immediate-abort",
    "-Zlocation-detail=none",
    "-Zfmt-debug=none",
]

```
