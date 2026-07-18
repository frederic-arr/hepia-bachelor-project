## Testing

> [!IMPORTANT] Running tests on WSL
> When running tests on WSL, use the `wsl` profile to skip tests that would fail
> due to WSL quirks:
> ```sh
> cargo nextest run --profile wsl
> ```
