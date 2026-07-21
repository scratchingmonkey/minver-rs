---
docs: true
title: Configuration
order: 4
---

Every calculation option has a default. You can override defaults with
environment variables, and command-line flags override environment variables.

Precedence, lowest to highest:

1. Built-in defaults
2. Environment variables (`TAGVER_*`)
3. Command-line flags

## Environment variables

Set any of these to configure tagver without passing flags — useful in CI where
the same settings apply to every invocation.

| Variable | Value | Description | Default |
| --- | --- | --- | --- |
| `TAGVER_TAGPREFIX` | string | Only consider tags with this prefix (e.g. `v`) | empty (all tags) |
| `TAGVER_AUTOINCREMENT` | `major` \| `minor` \| `patch` | Part to bump for the next version | `patch` |
| `TAGVER_DEFAULTPRERELEASEIDENTIFIERS` | dot-separated string | Default pre-release identifiers (e.g. `alpha.0`) | `alpha.0` |
| `TAGVER_MINIMUMMAJORMINOR` | `major.minor` | Floor the version at this `major.minor` (e.g. `1.0`) | none |
| `TAGVER_BUILDMETADATA` | string | Build metadata to append (after `+`) | none |
| `TAGVER_IGNOREHEIGHT` | `true` \| `false` | Do not append commit height to the version | `false` |
| `TAGVER_VERBOSITY` | `quiet` \| `normal` \| `verbose` \| `debug` \| `trace` | Log verbosity | `normal` |

Notes:

- An empty string is treated as "unset" for `TAGVER_TAGPREFIX`,
  `TAGVER_DEFAULTPRERELEASEIDENTIFIERS`, and `TAGVER_BUILDMETADATA`; the default
  is used instead.
- `TAGVER_IGNOREHEIGHT` must parse as a boolean (`true` or `false`).
- `verbose` and `info` are accepted as equivalents for `TAGVER_VERBOSITY`.

## Examples

```bash
# Configure once for the whole shell / CI job
export TAGVER_TAGPREFIX=v
export TAGVER_AUTOINCREMENT=minor
export TAGVER_MINIMUMMAJORMINOR=1.0

tagver --format json
```

A single flag still wins over the environment for that run:

```bash
export TAGVER_AUTOINCREMENT=minor
tagver --auto-increment patch   # this run bumps patch, not minor
```

## Defaults reference

With no configuration, tagver behaves as if:

- Working directory: current directory (`.`)
- Tag prefix: empty (all tags considered)
- Auto-increment: `patch`
- Default pre-release identifiers: `alpha.0`
- Minimum major.minor: none
- Build metadata: none
- Ignore height: `false`
- Verbosity: `normal`
