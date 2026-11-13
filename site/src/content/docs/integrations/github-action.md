---
title: "GitHub Action"
section: "Integrations"
order: 1
---

Run SpecSync in CI with zero setup. Auto-detects OS/arch, downloads the binary, runs validation.

---

## Basic Usage

```yaml
- uses: CorvidLabs/spec-sync@v6.3.0
  with:
    version: '6.3.0'
    strict: 'true'
    require-coverage: '100'
```

---

The stable GitHub `6.3.0` binary is published. The immutable `@v6.3.0` examples
explicitly select it because that tag's embedded default remains `6.2.0`.

## Inputs

| Input | Default | Description |
|:------|:--------|:------------|
| `version` | `6.3.0` | Release version to download. Pin an exact release for gates; `latest` follows the newest release and can change underneath you. The published `@v6.0.0` Action tag still defaults to `6.0.0-rc.14`; always pass `version: '6.0.0'` when using that tag. |
| `download-base-url` | `''` | Optional trusted release mirror URL for enterprise mirrors and release validation |
| `expected-sha256` | `''` | (after 6.3.0) Pinned SHA-256 of the release archive, one `<platform>=<sha256>` line per platform (`linux-x86_64`, `linux-aarch64`, `macos-x86_64`, `macos-aarch64`). When set, an archive whose digest differs, or a platform with no line, is refused before extraction |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage % (0–100) |
| `root` | `.` | Project root directory |
| `args` | `''` | Extra whitespace-separated CLI arguments passed to `specsync check`; shell quoting is not supported |
| `lifecycle-enforce` | `false` | Run lifecycle enforcement after the strict spec check |
| `require-archived` | `false` | Run the archive gate first. Fails before downloading or running anything when an SDD change under `.specsync/changes/` is still open. Repositories without `.specsync/changes/` pass. Needs `python3` on the runner |
| `archive-gate-only` | `false` | Run only the archive gate (implies `require-archived`) and skip the download, spec check and comment. Use it in a first job that every other job `needs:` |
| `comment` | `false` | Post spec drift results as a PR comment. Requires `pull_request` event and `pull-requests: write`. `@v6.3.0` and later fall back to the job summary on HTTP 403 (#886); immutable `@v6.2.0` and earlier do not |
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
      - uses: CorvidLabs/spec-sync@v6.3.0
        with:
          version: '6.3.0'
          strict: 'true'
          require-coverage: '100'
```

Release archives and their `.sha256` files are both fetched from the selected source. A missing or mismatched checksum fails before extraction. Treat `download-base-url` as a trust boundary and configure it only with an organization-controlled mirror.

Because the archive and its `.sha256` come from the same release, a release that replaced both would still pass that check. To pin the exact binary your workflow runs, commit its digest in the workflow with `expected-sha256`, one line per platform you run on, under the Action step's `with:`:

```text
with:
  version: "6.3.0"
  expected-sha256: |
    linux-x86_64=<sha256 of specsync-linux-x86_64.tar.gz>
    macos-aarch64=<sha256 of specsync-macos-aarch64.tar.gz>
```

`expected-sha256` is new after 6.3.0. The `@v6.3.0` Action does not have it, and GitHub ignores an input an Action does not declare with only an "Unexpected input" warning, so use it with an Action release that declares it.

A digest that differs, a platform with no line, a platform named twice or a malformed line each fail the step before anything is extracted.

Pin both the Action ref and its binary version. The YAML examples pin `@v6.3.0` with
`version: '6.3.0'`. This separately verified main wrapper defaults to the published
`6.3.0` binary; the immutable `@v6.3.0` tag retains default `6.2.0`.

```yaml
- uses: CorvidLabs/spec-sync@v6.3.0
  with:
    version: '6.3.0'
    strict: 'true'
```

Always pass `version: '6.0.0'` with `@v6.0.0`. That tag's composite Action still embeds
default `6.0.0-rc.14` (the last RC with assets when the tag was cut). Omitting `version` on
that tag downloads the RC binary. Immutable `@v6.3.0` retains default `6.2.0`;
explicitly pass `version: '6.3.0'` with it for the 6.3.0 stable binary. This later main
wrapper defaults to `6.3.0` after actual stable asset and supported-platform Action
checks. Floating `@v6` advances only after consumer checks of this exact wrapper
commit. Neither immutable `@v6.3.0` nor `@v6.0.0` is retargeted.

---

## Archive Gate: Nothing Runs Until Every Change Is Archived

Every SDD change must be archived (`specsync change ship`) before a pull request
merges. The archive gate makes that the first thing CI checks: one cheap job runs
the gate, and every other job depends on it, so a commit with an open change under
`.specsync/changes/` stops in seconds and nothing else runs.

```yaml
name: CI
on: [push, pull_request]

jobs:
  archive:
    name: Archive
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          ref: ${{ github.event_name == 'pull_request' && github.event.pull_request.head.sha || github.sha }}
      # The archive gate inputs are newer than v6.1.1, which ignores them. 6.1.2 is the
      # first release that includes them.
      - uses: CorvidLabs/spec-sync@v6.3.0
        with:
          version: '6.3.0'
          archive-gate-only: 'true'

  specsync:
    needs: archive
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          fetch-depth: 0
      - uses: CorvidLabs/spec-sync@v6.3.0
        with:
          version: '6.3.0'
          strict: 'true'
          require-coverage: '100'

  test:
    needs: archive
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - run: echo "your tests"
```

Make a job that depends on everything the required status check. GitHub treats a
skipped job as passing, so a required check on a job that `archive` skipped would
let the pull request merge. Use an aggregate job with `if: always()` that fails
unless `archive` and the others succeeded.

The gate needs no SpecSync binary. It prints the same sentence as the `Archive`
check, naming the change to archive first and the command to run (for example
`specsync change ship <id>`). The step also sets the `archived` output to `true` or
`false`.

To gate a single job instead, add `require-archived: 'true'` to an ordinary
invocation. The gate then runs before the download and the spec check.

The `@v6.1.1` and earlier tags predate these inputs. On those tags GitHub warns about
an unexpected input and skips nothing, so `archive-gate-only` would pass without
gating anything. Use `@v6.1.2` or later.

With the gate in place, the lifecycle runs locally up to ship: `change check
<id> --commit`, `fledge lanes run verify`, `change review`, then `change ship`.
Push the archived tip, and CI runs the full lane on it.

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
      - uses: CorvidLabs/spec-sync@v6.3.0
        with:
          version: '6.3.0'
          strict: 'true'
          comment: 'true'
```

**How it works:**
- Runs `specsync check --force` (always validates all specs — the hash cache is not committed to git)
- If `comment: 'true'`, also runs `specsync comment`
- Posts the markdown output as a PR comment (or updates an existing SpecSync comment)
- Requires `pull-requests: write` permission and the `pull_request` event trigger
- `@v6.3.0` and later ([#886](https://github.com/CorvidLabs/spec-sync/pull/886)) handle
  GitHub's HTTP 403 refusal (for example, a read-only fork PR token) by warning about the missing
  permission and writing the result to the job summary. The check result is unchanged; other
  comment errors fail the step. The fallback is in immutable `@v6.3.0`, used in the examples
  above; it is **not in `@v6.2.0` or earlier**, where a refused comment still fails the wrapper. Binary `version:` does not change
  the wrapper's behavior; its `uses:` revision does.

**Custom token (e.g., for private registries or cross-repo refs):**

```yaml
- uses: CorvidLabs/spec-sync@v6.3.0
  with:
    version: '6.3.0'
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
      - uses: CorvidLabs/spec-sync@v6.3.0
        with:
          version: '6.3.0'
          strict: 'true'
```

---

## Monorepo

```yaml
- uses: CorvidLabs/spec-sync@v6.3.0
  with:
    version: '6.3.0'
    root: './packages/backend'
    strict: 'true'
```

---

## Manual CI (without the action)

```yaml
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/download/v6.3.0/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

---

## Available Binaries

| Platform | Binary | Starts on |
|:---------|:-------|:----------|
| Linux x86_64 | `specsync-linux-x86_64` | glibc 2.35 or newer: Ubuntu 22.04, Debian 12 and newer |
| Linux x86_64 (static musl) | `specsync-linux-x86_64-musl` | Any x86_64 Linux, including Alpine |
| Linux aarch64 | `specsync-linux-aarch64` | glibc 2.35 or newer: Ubuntu 22.04, Debian 12 and newer |
| Linux aarch64 (static musl) | `specsync-linux-aarch64-musl` | Any aarch64 Linux, including Alpine |
| macOS x86_64 | `specsync-macos-x86_64` | |
| macOS aarch64 (Apple Silicon) | `specsync-macos-aarch64` | |

The Action downloads the glibc build for the runner's architecture. Every GitHub-hosted Ubuntu
image, 22.04 or newer on x86_64 and arm64, is at or above the floor.

Below the floor, a glibc download stops before it prints anything, with
`version 'GLIBC_2.xx' not found`. This can happen on a self-hosted runner, an older container or
a workstation; use the musl download for your architecture there. Every release runs each Linux
download on Ubuntu 22.04 and Debian 12, and the musl ones also on Debian 11 and Alpine, before
publishing it.

Releases up to and including 6.2.0 predate this floor
([#882](https://github.com/CorvidLabs/spec-sync/issues/882)). Their glibc downloads need glibc
2.39, so they do not start on `ubuntu-22.04` or `ubuntu-22.04-arm` runners or on Debian 12, and
they have no aarch64 musl download. On those versions use `specsync-linux-x86_64-musl`, a newer
runner image, or `cargo install specsync --locked`.

No Windows binary is published, and CI does not build or test Windows (#735). On a Windows
workstation, run SpecSync in WSL 2, or build it from source with `cargo install specsync --locked`.
