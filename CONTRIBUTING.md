# Contributing to adbrose

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Clone and build
git clone https://github.com/thebennies/adbrose.git
cd adbrose
cargo build

# Run tests
cargo test

# Run with logging
cargo run -- --log-file /tmp/adbrose.log

# Lint
cargo clippy -- -D warnings
cargo fmt --all -- --check
```

## Pull Request Process

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/my-feature` or `fix/my-fix`
3. **Make your changes** — follow the existing code style
4. **Add tests** if your change is testable
5. **Run the checks** — `cargo test`, `cargo clippy`, `cargo fmt`
6. **Commit** with clear, descriptive messages
7. **Open a PR** against `main`

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- Resolve all clippy warnings (`cargo clippy`)
- Keep modules focused — each file in `src/` has a single responsibility
- Use `thiserror` for library errors, `anyhow` for application-level errors

## Reporting Bugs

Open a [GitHub issue](https://github.com/thebennies/adbrose/issues) with:

- Your OS and terminal
- Steps to reproduce
- Expected vs actual behavior
- Any relevant log output (use `--log-file`)

## Versioning

This project uses [CalVer](https://calver.org/) (`YYYY.MM.MICRO`). Changes are tracked in [CHANGELOG.md](CHANGELOG.md).

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
