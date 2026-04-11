---
title: GitHub Action
layout: default
nav_order: 6
---

# GitHub Action
{: .no_toc }

Run SpecSync in CI with zero setup. Auto-detects OS/arch, downloads the binary, runs validation.
{: .fs-6 .fw-300 }

<details open markdown="block">
  <summary>Table of contents</summary>
  {: .text-delta }
1. TOC
{:toc}
</details>

---

## Basic Usage

```yaml
- uses: CorvidLabs/spec-sync@v3
  with:
    strict: 'true'
    require-coverage: '100'
```

---

## Inputs

| Input | Default | Description |
|:------|:--------|:------------|
| `version` | `latest` | Release version to download |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage % (0–100) |
| `root` | `.` | Project root directory |
| `args` | `''` | Extra CLI arguments passed to `specsync check` |
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
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v3
        with:
          strict: 'true'
          require-coverage: '100'
```

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
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v3
        with:
          strict: 'true'
          comment: 'true'
```

**How it works:**
- Runs `specsync check` as normal
- If `comment: 'true'`, also runs `specsync diff --format markdown`
- Posts the markdown output as a PR comment (or updates an existing SpecSync comment)
- Requires `pull-requests: write` permission and the `pull_request` event trigger

**Custom token (e.g., for private registries or cross-repo refs):**

```yaml
- uses: CorvidLabs/spec-sync@v3
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
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v3
        with:
          strict: 'true'
```

---

## Monorepo

```yaml
- uses: CorvidLabs/spec-sync@v3
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
