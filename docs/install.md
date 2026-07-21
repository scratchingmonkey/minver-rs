---
docs: true
title: Installation
order: 2
install: true
---

tagver is a single statically linked binary with no runtime dependencies.
Choose whichever method fits your platform.

## Cargo (crates.io)

The CLI is published as `tagver-cli` and installs a binary named `tagver`.

```bash
cargo install tagver-cli
```

The library crate is published separately as `tagver` if you want to embed
version calculation in your own Rust program:

```bash
cargo add tagver
```

## Homebrew

tagver is available from the Wyrd Company Homebrew tap.

```bash
brew install wyrd-company/tools/tagver
```

Or tap first, then install by short name:

```bash
brew tap wyrd-company/tools
brew install tagver
```

## Prebuilt release binaries

Each release ships prebuilt archives on GitHub. Download the one for your
platform from the [latest release](https://github.com/wyrd-company/tagver/releases/latest):

| Platform | Asset |
| --- | --- |
| Linux x86_64 | `tagver-linux-x86_64.tar.gz` |
| Linux arm64 | `tagver-linux-arm64.tar.gz` |
| macOS arm64 (Apple Silicon) | `tagver-macos-arm64.tar.gz` |
| Windows x86_64 | `tagver-windows-x86_64.zip` |

Each archive contains a directory holding the `tagver` binary
(`tagver.exe` on Windows). Extract it and place the binary on your `PATH`:

```bash
# Linux x86_64 example
curl -LO https://github.com/wyrd-company/tagver/releases/latest/download/tagver-linux-x86_64.tar.gz
tar -xzf tagver-linux-x86_64.tar.gz
sudo install tagver-linux-x86_64/tagver /usr/local/bin/tagver
```

## Build from source

Building requires a Rust toolchain at or above the minimum supported Rust
version (MSRV) of **1.75**.

```bash
git clone https://github.com/wyrd-company/tagver.git
cd tagver
cargo build --release --workspace
# binary at target/release/tagver
```

To install it directly onto your `PATH` from a local checkout:

```bash
cargo install --path crates/cli
```

## Verify

Confirm the install and check the build details:

```bash
tagver --version
tagver --help
```
