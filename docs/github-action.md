---
title: GitHub Action
layout: default
nav_order: 5
---

# GitHub Action
{: .no_toc }

Run SpecSync in CI with zero setup. The action auto-detects your runner's OS and architecture, downloads the correct binary, and runs validation.
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
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

---

## Inputs

| Input | Default | Description |
|:------|:--------|:------------|
| `version` | `latest` | SpecSync release version to download |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage percentage (0-100) |
| `root` | `.` | Project root directory |
| `args` | `''` | Additional CLI arguments passed to `specsync check` |

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
      - uses: CorvidLabs/spec-sync@v1
        with:
          strict: 'true'
          require-coverage: '100'
```

---

## Multi-Platform Matrix

Test across operating systems:

```yaml
jobs:
  specsync:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v1
        with:
          strict: 'true'
```

---

## Monorepo Usage

Run against a specific package:

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    root: './packages/backend'
    strict: 'true'
```

---

## Manual CI Setup

If you prefer not to use the action, install the binary directly:

```yaml
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

---

## Platform Binaries

The action and release workflow provide binaries for:

| Platform | Binary |
|:---------|:-------|
| Linux x86_64 | `specsync-linux-x86_64` |
| Linux aarch64 | `specsync-linux-aarch64` |
| macOS x86_64 | `specsync-macos-x86_64` |
| macOS aarch64 (Apple Silicon) | `specsync-macos-aarch64` |
| Windows x86_64 | `specsync-windows-x86_64.exe` |
