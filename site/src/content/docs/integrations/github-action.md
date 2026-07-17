---
title: "GitHub Action"
section: "Integrations"
order: 1
---

Run SpecSync in CI with zero setup. Auto-detects OS/arch, downloads the binary, runs validation.

---

## Basic Usage

```yaml
- uses: CorvidLabs/spec-sync@v5.1.1
  with:
    strict: 'true'
    require-coverage: '100'
```

---

## Inputs

| Input | Default | Description |
|:------|:--------|:------------|
| `version` | `5.1.1` | Release version to download; set `latest` to follow the newest release |
| `download-base-url` | `''` | Optional trusted release mirror URL for enterprise mirrors and release validation |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage % (0–100) |
| `root` | `.` | Project root directory |
| `args` | `''` | Extra whitespace-separated CLI arguments passed to `specsync check`; shell quoting is not supported |
| `lifecycle-enforce` | `false` | Run lifecycle enforcement after the strict spec check |
| `comment` | `false` | Post spec drift results as a PR comment. Requires `pull_request` event and write permissions |
| `token` | `${{ github.token }}` | GitHub token for posting PR comments. Override if using a PAT for cross-repo access |

---

## Full Workflow

```yaml
name: Spec Check
on: [push, pull_request]

jobs:
  specsync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5.1.1
        with:
          strict: 'true'
          require-coverage: '100'
```

Release archives and their `.sha256` files are both fetched from the selected source. A missing or mismatched checksum fails before extraction. Treat `download-base-url` as a trust boundary and configure it only with an organization-controlled mirror.

Use the immutable release ref until compatible-channel promotion is complete, and pin both the
Action ref and its binary version:

```yaml
- uses: CorvidLabs/spec-sync@v5.1.1
  with:
    version: '5.1.1'
    strict: 'true'
```

After the exact-version Action passes supported Linux, macOS, and Windows smoke tests, the floating
`v5` ref is promoted as the compatible 5.x channel and may advance to newer verified 5.x releases.

---

## PR Comments

Post spec drift results directly on pull requests. SpecSync runs `diff --format markdown` and posts (or updates) a comment showing added/removed exports.

```yaml
name: Spec Check
on:
  pull_request:
    types: [opened, synchronize]

jobs:
  specsync:
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5.1.1
        with:
          strict: 'true'
          comment: 'true'
```

**How it works:**
- Runs `specsync check --force` (always validates all specs — the hash cache is not committed to git)
- If `comment: 'true'`, also runs `specsync diff --format markdown`
- Posts the markdown output as a PR comment (or updates an existing SpecSync comment)
- Requires `pull-requests: write` permission and the `pull_request` event trigger

**Custom token (e.g., for private registries or cross-repo refs):**

```yaml
- uses: CorvidLabs/spec-sync@v5.1.1
  with:
    comment: 'true'
    token: ${{ secrets.MY_PAT }}
```

---

## Multi-Platform Matrix

```yaml
jobs:
  specsync:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v5.1.1
        with:
          strict: 'true'
```

---

## Monorepo

```yaml
- uses: CorvidLabs/spec-sync@v5.1.1
  with:
    root: './packages/backend'
    strict: 'true'
```

---

## Manual CI (without the action)

```yaml
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

---

## Available Binaries

| Platform | Binary |
|:---------|:-------|
| Linux x86_64 | `specsync-linux-x86_64` |
| Linux aarch64 | `specsync-linux-aarch64` |
| macOS x86_64 | `specsync-macos-x86_64` |
| macOS aarch64 (Apple Silicon) | `specsync-macos-aarch64` |
| Windows x86_64 | `specsync-windows-x86_64.exe` |
