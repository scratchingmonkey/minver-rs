# Contributing

## Development

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p minver-cli

# Build release version
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p minver-core

# Run integration tests
cargo test --test integration
```

### Example repositories

To test the implementation, you can create example Git repositories:

```bash
# Linear history with tags
git init linear-test
cd linear-test
git commit --allow-empty -m "Initial commit"
git tag 1.0.0
git commit --allow-empty -m "Second commit"
git commit --allow-empty -m "Third commit"

# Test with MinVer
minver --working-directory linear-test
```

## Code Style

- Run `cargo fmt --all` before committing
- Ensure `cargo clippy --workspace --all-targets -- -D warnings` passes
- Follow Rust naming conventions and prefer small, focused functions

## Pull Request Process

1. Fork the repository and create a feature branch (`git checkout -b feature/my-feature`)
2. Make your changes with appropriate tests
3. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`
4. Ensure commits are focused and well-described
5. Open a pull request with a clear summary and testing notes

## Reporting Issues

When filing an issue, please include:
- minver-rs version (`minver --version`)
- Rust version (`rustc --version`)
- Operating system and shell
- Reproduction steps and expected vs. actual behavior
