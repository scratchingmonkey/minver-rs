---
docs: true
title: GitHub Action
order: 5
---

tagver ships a composite GitHub Action that downloads the prebuilt `tagver`
binary from releases, runs it in your checkout, and exposes the calculated
version and its components as step outputs.

Reference it as `wyrd-company/tagver@v0`.

## Inputs

All inputs are optional.

| Input | Description | Default |
| --- | --- | --- |
| `tag-prefix` | Tag prefix to filter tags (e.g. `v` for `v1.0.0`) | none |
| `auto-increment` | Auto-increment policy: `major`, `minor`, `patch` | none |
| `default-pre-release-identifiers` | Default pre-release identifiers (e.g. `alpha.0`) | none |
| `minimum-major-minor` | Minimum `major.minor` version constraint (e.g. `1.0`) | none |
| `build-metadata` | Build metadata to append to versions | none |
| `ignore-height` | Ignore height in version calculation | `false` |
| `working-directory` | Working directory to analyze | `.` |
| `tagver-version` | Version of tagver to download (e.g. `v0.1.0`, or `latest`) | `latest` |

## Outputs

| Output | Description |
| --- | --- |
| `version` | The full calculated version |
| `major` | The major version component |
| `minor` | The minor version component |
| `patch` | The patch version component |
| `pre-release` | The pre-release component (if any) |
| `build-metadata` | The build metadata component (if any) |

## Usage

Check out the full history (`fetch-depth: 0`) so tagver can see your tags and
commit height, then read the outputs from the step's `id`.

```yaml
steps:
  - uses: actions/checkout@v4
    with:
      fetch-depth: 0

  - name: Calculate version
    id: tagver
    uses: wyrd-company/tagver@v0
    with: # all optional
      tag-prefix: 'v'
      auto-increment: patch
      default-pre-release-identifiers: 'alpha.0'
      minimum-major-minor: '1.0'
      build-metadata: ci
      ignore-height: false
      working-directory: '.'
      tagver-version: latest

  - name: Use version
    run: |
      echo "Full:           ${{ steps.tagver.outputs.version }}"
      echo "Major:          ${{ steps.tagver.outputs.major }}"
      echo "Minor:          ${{ steps.tagver.outputs.minor }}"
      echo "Patch:          ${{ steps.tagver.outputs.patch }}"
      echo "Pre-release:    ${{ steps.tagver.outputs.pre-release }}"
      echo "Build-metadata: ${{ steps.tagver.outputs.build-metadata }}"
```

> The Action runs on Linux, macOS, and Windows runners. It selects the matching
> release asset for the runner's OS and architecture, so `fetch-depth: 0` on
> `actions/checkout` is the only requirement for accurate height calculation.
