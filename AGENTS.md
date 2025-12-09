# Agent Instructions for Tagver

This is a small Rust workspace with two main crates:

- `crates/core` — core library implementing calculation/testable logic. This is the domain logic you should prefer changing.
- `crates/cli` — command-line front-end that wires CLI args/env to the core library as well as packaging/build/install logic.

It also includes a GitHub action defined in `action.yml` that downloads release artifacts, runs TagVer, and exposes outputs.

## Build / test / release workflows

- Humans use the devcontainer defined in `.devcontainer/` for consistency. Changes to the dev environment should be made there.
- Dev: `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --all`, and `cargo clippy --workspace --all-targets -- -D warnings`. IMPORTANT: No task is completed until all pass.
  - Prefer changes in `crates/core` for algorithmic or semantic-versioning logic; keep CLI-only changes in `crates/cli`.
  - Keep artifact names in sync between `release.yml` and `action.yml`.
  - Use `cargo add` to add dependencies to ensure version consistency across the workspace.
  - `action.yml` (root) is a composite action that downloads the release archive, extracts the binary, runs it with `--format json`, and parses the JSON to set outputs. If you change artifact naming or JSON shape, update `action.yml` accordingly. The CLI integration test `crates/cli/tests/cli.rs::test_json_output` validates the JSON shape from `--format json`
- CI: `.github/workflows/ci.yml`:
  - Runs on every PR and push to `main`.
  - Checks out code, sets up Rust, builds, formats, lints, and tests the entire workspace including the GitHub action.
- Release: `.github/workflows/release.yml`:
  - Triggers only after a `N.N.N` tag is pushed.
  - Builds multiple platform artifacts using `cross` for non-native targets, packages them to `dist/<artifact>` and uploads; keep the `matrix.include` entries in `release.yml` consistent with binary paths used when packaging.
  - Updates crate versions in `crates/*/Cargo.toml` using `tagver`, publishes to crates.io and updates a major `vX` tag after release.

## Conventions

- Env var names use a `TAGVER_` prefix (e.g. `TAGVER_TAGPREFIX`, `TAGVER_IGNOREHEIGHT`) mirroring all of the CLI flags except `--format`.
- Tests that operate on Git repositories create temporary repos with `git init` and require `fetch-depth: 0` when running in CI.

## Useful commands

- Build: `cargo build --workspace --release`
- Run CLI locally: `cargo run --bin tagver -- --format json`
- Run tests: `cargo test --workspace` (or `cargo test --test cli` for integration tests)
- Format & lint: `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings`

## Navigation

All of the relevant files in the repo are listed here:

```
├── action.yml # GitHub Action definition
├── AGENTS.md # Agent instructions (this file)
├── Cargo.toml # Cargo workspace definition
├── crates # Workspace crates
│   ├── cli # Command-line interface crate
│   │   ├── build.rs # Build script for embedding version info
│   │   ├── Cargo.toml # CLI crate manifest
│   │   ├── src
│   │   │   └── main.rs # CLI entry point
│   │   └── tests
│   │       └── cli.rs # Integration tests for CLI
│   └── core # Core library crate
│       ├── benches # Benchmarking crate
│       │   └── version_calculation.rs # Benchmark for version calculation
│       ├── Cargo.toml # Core crate manifest
│       ├── src
│       │   ├── config.rs # Configuration handling
│       │   ├── error.rs # Error definitions
│       │   ├── git.rs # Git repository interactions
│       │   ├── lib.rs # Core library entry point
│       │   ├── tags.rs # Tag discovery and parsing
│       │   └── version.rs # Version calculation logic
│       └── tests
│           ├── auto_increment.rs # Tests for auto-increment logic
│           ├── build_metadata.rs # Tests for build metadata handling
│           ├── common
│           │   ├── fixtures.rs # Common test fixtures
│           │   ├── git.rs # Common Git test utilities
│           │   └── mod.rs # Common test module
│           ├── default_prerelease.rs # Tests for default pre-release identifiers
│           ├── fixtures # Test fixtures directory
│           │   ├── log.0.0.txt
│           │   ├── log.2.0.txt
│           │   ├── log.3.0.txt
│           │   └── versions.txt
│           ├── log_messages.rs # Tests for log message generation
│           ├── min_major_minor.rs # Tests for minimum major.minor handling
│           ├── tag_prefixes.rs # Tests for tag prefix handling
│           └── versions.rs # Tests for version calculation
├── .devcontainer
│   └── devcontainer.json # Devcontainer configuration
├── .github
│   └── workflows
│       ├── ci.yml # CI workflow
│       └── release.yml # Release workflow
├── .gitignore
└── README.md
```

TRUST THESE INSTRUCTIONS AND ONLY SEARCH IF INFORMATION IS MISSING OR INCORRECT.
