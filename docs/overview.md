---
docs: true
title: Overview
order: 1
---

tagver uses Git tags as the single source of truth for versions and calculates
distance from the tagged commit to generate alpha versions. It is a single
statically linked binary with no runtime dependencies.

It reads your Git history and computes a SemVer 2.0.0 version from the nearest
tag plus commit height. Merge commits are handled cleanly via first-parent
traversal, and output is available as JSON for scripting.

![tagver command-line demo](demo.gif)

tagver is inspired by [MinVer](https://github.com/adamralph/minver) and built
on the pure-Rust [gitoxide](https://github.com/GitoxideLabs/gitoxide) Git
implementation.

## Features

| Feature | What it gives you |
| --- | --- |
| Tag-driven versioning | Git tags are the single source of truth |
| Height calculation | Commit distance from the nearest tag generates alpha versions |
| Cross-platform | One binary, wherever Rust compiles |
| Zero dependencies | Statically linked, nothing to install at runtime |
| JSON output | Machine-readable version details for scripting |
| First-parent traversal | Merge commits handled correctly |
| Strict SemVer 2.0.0 | Versions parse and render per the SemVer specification |
| GitHub Action | Exposes the calculated version and components as workflow outputs |

## How it works

tagver walks your commit history and synthesizes a version:

1. **Tag discovery** — find all Git tags matching the configured prefix.
2. **Version parsing** — parse those tags as SemVer 2.0.0 versions.
3. **Commit traversal** — walk from `HEAD` (first-parent) to the nearest
   tagged ancestor.
4. **Height calculation** — count the commits between `HEAD` and that base tag.
5. **Version synthesis** — if `HEAD` is exactly on a tag, use it as-is;
   otherwise apply the auto-increment policy, append the default pre-release
   identifiers and the height, honor any minimum major.minor constraint, and
   append build metadata if provided.

### Example results

| Git state | Result |
| --- | --- |
| On tag `1.0.0` | `1.0.0` |
| 5 commits after `1.0.0` | `1.0.1-alpha.0.5` |
| On tag `1.0.0-beta.1` | `1.0.0-beta.1` |
| 3 commits after `1.0.0-beta.1` | `1.0.0-beta.1.3` |
| No tags | `0.0.0-alpha.0` |
| 2 commits from root | `0.0.0-alpha.0.2` |
