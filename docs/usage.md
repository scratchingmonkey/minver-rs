---
docs: true
title: Usage
order: 3
---

Run tagver anywhere in a Git repository to print the calculated version, or ask
for JSON when a script needs to read it.

```bash
tagver                 # print the calculated version (text)
tagver --format json   # machine-readable output for CI
tagver --help          # print all command-line options
tagver --version       # print the version and build details
```

## Options

The first positional argument is the working directory to analyze; it defaults
to the current directory (`.`).

| Flag | Description | Default |
| --- | --- | --- |
| `-t, --tag-prefix <PREFIX>` | Only consider tags with this prefix (e.g. `release-`) | empty (all tags) |
| `-a, --auto-increment <PART>` | Part to bump for the next version: `major`, `minor`, `patch` | `patch` |
| `-p, --default-pre-release-identifiers <IDS>` | Dot-separated pre-release identifiers (e.g. `alpha.0`) | `alpha.0` |
| `-m, --minimum-major-minor <MAJOR.MINOR>` | Floor the version at this `major.minor` (e.g. `1.0`) | none |
| `-b, --build-metadata <META>` | Build metadata to append (after `+`) | none |
| `-i, --ignore-height` | Do not append commit height to the version | off |
| `-f, --format <FORMAT>` | Output format: `text` or `json` | `text` |
| `-v, --verbosity <LEVEL>` | Log verbosity: `quiet`, `normal`, `verbose`, `debug`, `trace` | `normal` |

The calculation flags and `--verbosity` have `TAGVER_*` environment-variable
equivalents. The working-directory positional argument and `--format` do not.
See [Configuration](config).

## JSON output

`--format json` prints a pretty-printed object with the full version and its
components, ready to parse in a script:

```bash
tagver --format json
```

```json
{
  "version": "1.0.1-alpha.0.5",
  "major": 1,
  "minor": 0,
  "patch": 1,
  "pre_release": ["alpha", "0", "5"],
  "build_metadata": null,
  "height": 5,
  "is_from_tag": false
}
```

- `version` — the full calculated SemVer string.
- `major`, `minor`, `patch` — numeric version components.
- `pre_release` — an array of pre-release identifiers (empty when none).
- `build_metadata` — the build metadata, or `null` when none.
- `height` — commits between `HEAD` and the base tag.
- `is_from_tag` — `true` when `HEAD` sits exactly on a tag.

## Examples

```bash
# Only tags like release-1.2.3, bump minor for the next version
tagver --tag-prefix release- --auto-increment minor

# Floor at 2.0 and stamp build metadata
tagver --minimum-major-minor 2.0 --build-metadata ci.42

# Analyze a different repository directory
tagver ../other-repo --format json
```
