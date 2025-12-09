# TagVer

[![CI](https://github.com/scratchingmonkey/tagver/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/scratchingmonkey/tagver/actions/workflows/ci.yml?query=branch%3Amain) [![Crates.io Version](https://img.shields.io/crates/v/tagver)](https://crates.io/crates/tagver-cli) [![docs.rs](https://img.shields.io/docsrs/tagver?logo=docsdotrs&link=https%3A%2F%2Fdocs.rs%2Ftagver%2Flatest%2Ftagver%2F)](https://crates.io/crates/tagver-cli)
 ![Binary Size](https://img.shields.io/crates/size/tagver-cli?logo=rust) ![MSRV](https://img.shields.io/crates/msrv/tagver?logo=rust)


A fast, dependency-free, CLI tool and Rust library for calculating version numbers from Git repository tags.

This project is heavily influenced by the excellent [MinVer CLI](https://github.com/adamralph/minver) CLI .NET tool, written in Rust and incorporating the [`gitoxide`](https://github.com/GitoxideLabs/gitoxide) crate.

## Features

- **Tag-driven versioning**: Uses Git tags as the single source of truth for versions
- **Height calculation**: Automatically calculates distance from tagged commits to generate alpha versions
- **Cross-platform**: Single binary that runs anywhere Rust compiles
- **Zero dependencies**: Statically linked binary with no runtime dependencies
- **Environment variable support**: Fully configurable using environment variables
- **JSON formatted version output**: Outputs version information to JSON for easy scripting
- **First-parent traversal**: Correctly handles merge commits seamlessly
- **Semantic versioning**: Strict adherence to SemVer 2.0.0 specification
- **A GitHub Action**: Provides version information as outputs automatically

## Requirements

- Rust 1.75 or newer (MSRV)

## Installation

### From crates.io

```bash
cargo install tagver-cli
```

## Usage

### Basic usage

```bash
# Calculate version for current repository
tagver

# With custom tag prefix
tagver --tag-prefix v

# Ignore height (use exact tag version)
tagver --ignore-height

# Print all command-line options
tagver --help

# Output JSON
tagver --format JSON
```

### Environment variables

Most options can also be set via environment variables:

- `TAGVER_TAGPREFIX`
- `TAGVER_AUTOINCREMENT`
- `TAGVER_DEFAULTPRERELEASEIDENTIFIERS`
- `TAGVER_MINIMUMMAJORMINOR`
- `TAGVER_IGNOREHEIGHT`
- `TAGVER_BUILDMETADATA`
- `TAGVER_VERBOSITY`

## How it works

TagVer follows the same algorithm as the original MinVer:

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

## GitHub Action

This repository also ships a lightweight GitHub Action that downloads the pre-built `tagver` binary from releases and exposes the calculated version and components as outputs. It makes it easy to use TagVer in workflows without installing the CLI on the runner.

Usage example:

```yaml
steps:
   - uses: actions/checkout@v4
      with:
         fetch-depth: 0

   - name: Calculate version
      id: tagver
      uses: scratchingmonkey/tagver@v0
      with:
         tag-prefix: 'v'

   - name: Use version
      run: |
         echo "Full: ${{ steps.tagver.outputs.version }}"
         echo "Major: ${{ steps.tagver.outputs.major }}"
         echo "Pre-release: ${{ steps.tagver.outputs.pre-release }}"
```

Inputs mirror the CLI options (for example `tag-prefix`, `auto-increment`, `ignore-height`, etc.). The action provides the following outputs:

- `version`: full SemVer string
- `major`, `minor`, `patch`: numeric components
- `pre-release`: prerelease identifiers joined by `.`
- `build-metadata`: build metadata string (empty if none)

## Acknowledgments

- [Adam Ralph](https://github.com/adamralph) for creating the original MinVer
- [Gitoxide](https://github.com/GitoxideLabs/gitoxide) for the excellent pure-Rust Git implementation
- The Rust community for the amazing ecosystem of libraries
