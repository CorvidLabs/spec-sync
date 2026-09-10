---
title: "GitHub Action"
section: "Integrations"
order: 1
---

Run SpecSync in CI with zero setup. Auto-detects OS/arch, downloads the binary, runs validation.

---

## Basic Usage

```yaml
- uses: CorvidLabs/spec-sync@v6.0.0
  with:
    version: '6.0.0'
    strict: 'true'
    require-coverage: '100'
```

---

## Inputs

| Input | Default | Description |
|:------|:--------|:------------|
| `version` | `6.0.0` | Release version to download. Pin an exact release for gates; `latest` follows the newest release and can change underneath you. The published `@v6.0.0` Action tag still defaults to `6.0.0-rc.14`; always pass `version: '6.0.0'` when using that tag. |
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
      - uses: CorvidLabs/spec-sync@v6.0.0
        with:
          version: '6.0.0'
          strict: 'true'
          require-coverage: '100'
```

Release archives and their `.sha256` files are both fetched from the selected source. A missing or mismatched checksum fails before extraction. Treat `download-base-url` as a trust boundary and configure it only with an organization-controlled mirror.

Pin both the Action ref and its binary version. There is no floating `v6` tag yet.

```yaml
- uses: CorvidLabs/spec-sync@v6.0.0
  with:
    version: '6.0.0'
    strict: 'true'
```

Always pass `version: '6.0.0'` with `@v6.0.0`. That tag's composite Action still embeds
default `6.0.0-rc.14` (the last RC with assets when the tag was cut). Omitting `version`
downloads the RC binary. `main` now defaults to `6.0.0`; a later Action tag will make omit
safe. After Linux and macOS smoke tests, a floating `v6` ref may be promoted by hand as the
compatible 6.x channel. Do not invent that tag.

---

## PR Comments

Post spec drift results directly on pull requests. SpecSync runs `specsync comment` and posts (or updates) a check-summary comment.

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
      - uses: CorvidLabs/spec-sync@v6.0.0
        with:
          version: '6.0.0'
          strict: 'true'
          comment: 'true'
```

**How it works:**
- Runs `specsync check --force` (always validates all specs — the hash cache is not committed to git)
- If `comment: 'true'`, also runs `specsync comment`
- Posts the markdown output as a PR comment (or updates an existing SpecSync comment)
- Requires `pull-requests: write` permission and the `pull_request` event trigger

**Custom token (e.g., for private registries or cross-repo refs):**

```yaml
- uses: CorvidLabs/spec-sync@v6.0.0
  with:
    version: '6.0.0'
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
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v6.0.0
        with:
          version: '6.0.0'
          strict: 'true'
```

---

## Monorepo

```yaml
- uses: CorvidLabs/spec-sync@v6.0.0
  with:
    version: '6.0.0'
    root: './packages/backend'
    strict: 'true'
```

---

## Manual CI (without the action)

```yaml
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/download/v6.0.0/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

---

## Available Binaries

| Platform | Binary |
|:---------|:-------|
| Linux x86_64 | `specsync-linux-x86_64` |
| Linux x86_64 (static musl, any distro) | `specsync-linux-x86_64-musl` |
| Linux aarch64 | `specsync-linux-aarch64` |
| macOS x86_64 | `specsync-macos-x86_64` |
| macOS aarch64 (Apple Silicon) | `specsync-macos-aarch64` |

Windows is not a supported target as of 6.0 — no Windows binary is published. Run SpecSync
under WSL, or build it from source with `cargo install specsync`.
