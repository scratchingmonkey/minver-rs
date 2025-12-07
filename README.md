# MinVer-RS

[![CI](https://github.com/scratchingmonkey/minver-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/scratchingmonkey/minver-rs/actions/workflows/ci.yml?query=branch%3Amain) [![MSRV 1.75+](https://img.shields.io/badge/MSRV-1.75%2B-blue.svg)](https://blog.rust-lang.org/2023/12/07/Rust-1.75.0.html)

A Rust implementation of the [MinVer CLI](https://github.com/adamralph/minver) - minimalistic versioning using Git tags.

This project ports the excellent MinVer CLI .NET tool to Rust and incorporates the [`gitoxide`](https://github.com/GitoxideLabs/gitoxide) crate, providing a fast, dependency-free CLI tool and library for calculating version numbers from Git repository tags.

## Features

- **Tag-driven versioning**: Uses Git tags as the single source of truth for versions
- **Height calculation**: Automatically calculates distance from tagged commits
- **Cross-platform**: Single binary that runs anywhere Rust compiles
- **Zero dependencies**: Statically linked binary with no runtime dependencies
- **Environment variable support**: Full compatibility with existing MinVer CLI workflows
- **First-parent traversal**: Correctly handles merge commits like the original
- **Semantic versioning**: Strict adherence to SemVer 2.0.0 specification

## Requirements

- Rust 1.75 or newer (MSRV)

## Installation

### From crates.io (recommended)

```bash
cargo install minver-rs-cli
```

### Pre-built binaries

Download platform-specific archives from [GitHub Releases](https://github.com/scratchingmonkey/minver-rs/releases):
- `minver-linux-x86_64.tar.gz` (Linux x86_64)
- `minver-macos-arm64.tar.gz` (macOS Apple Silicon)
- `minver-windows-x86_64.zip` (Windows x86_64)

### From source

```bash
git clone https://github.com/scratchingmonkey/minver-rs
cd minver-rs
cargo install --path crates/cli
```

## Usage

### Basic usage

```bash
# Calculate version for current repository
minver

# With custom tag prefix
minver --tag-prefix v

# Ignore height (use exact tag version)
minver --ignore-height

# Print all command-line options
minver --help
```

### Example

```bash
$ git tag 1.2.3

$ minver
1.2.3
```

### Environment variables

All options can also be set via environment variables:

- `MINVERTAGPREFIX`
- `MINVERAUTOINCREMENT`
- `MINVERDEFAULTPRERELEASEIDENTIFIERS`
- `MINVERMINIMUMMAJORMINOR`
- `MINVERIGNOREHEIGHT`
- `MINVERBUILDMETADATA`
- `MINVERVERBOSITY`

## How it works

MinVer-RS follows the same algorithm as the original MinVer:

1. **Tag discovery**: Find all Git tags that match the configured prefix
2. **Version parsing**: Parse tags as semantic versions (SemVer 2.0.0)
3. **Commit traversal**: Walk the commit graph from HEAD to find the nearest tagged ancestor
4. **Height calculation**: Count commits between current position and the base tag
5. **Version synthesis**: 
   - If at exact tag: use version as-is
   - If not at tag: apply auto-increment, add pre-release identifiers, append height
   - Apply minimum major.minor constraint if configured
   - Append build metadata if provided

### Version calculation examples

| Git state | Result |
|-----------|--------|
| On tag `1.0.0` | `1.0.0` |
| 5 commits after `1.0.0` | `1.0.1-alpha.0.5` |
| On tag `1.0.0-beta.1` | `1.0.0-beta.1` |
| 3 commits after `1.0.0-beta.1` | `1.0.0-beta.1.3` |
| No tags | `0.0.0-alpha.0` |
| 2 commits from root | `0.0.0-alpha.0.2` |

## Comparison with original MinVer

| Feature | MinVer (.NET) | MinVer-RS (Rust) |
|---------|---------------|------------------|
| Language | C# | Rust |
| Distribution | NuGet package | Static binary |
| Startup time | ~200ms | <10ms |
| Dependencies | .NET SDK | None |
| Cross-platform | .NET supported | All Rust targets |
| Environment variables | ✅ | ✅ |
| CLI interface | ✅ | ✅ |
| Library usage | ✅ | ✅ |

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Adam Ralph](https://github.com/adamralph) for creating the original MinVer
- [Gitoxide](https://github.com/GitoxideLabs/gitoxide) for the excellent pure-Rust Git implementation
- The Rust community for the amazing ecosystem of libraries
