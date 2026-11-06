# Lesson bundle: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Build the Linux glibc release binaries on Ubuntu 22.04, add a static aarch64 musl fallback, and stop a release whose Linux download would not start on the documented glibc 2.35 baseline
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/release.yml, .github/workflows/rc-assets.yml, .github/workflows/ci.yml, .github/scripts/linux-baseline.py, .github/scripts/test-linux-baseline.py, .github/scripts/validate-release-candidate.py, .github/scripts/test-validate-release-candidate.py, README.md, site/src/content/docs/integrations/github-action.md, docs/RELEASING.md, docs/HLD.md, SCOPE.md, CHANGELOG.md, fledge.toml, specs/github/, hi/install.md, INTENT.md, .chough/, .github/scripts/test-required-ci-gate.py, site/src/content/docs/quickstart.md
- **Acceptance**: The `github` spec gains REQ-github-025: the prebuilt Linux release downloads start on the documented glibc 2.35 baseline (Ubuntu 22.04, Debian 12 and newer), a static musl download per architecture starts below it, and both release workflows refuse to upload a Linux archive that does not start on its baseline; the spec version, Public API notes and Change Log follow.
- **Acceptance**: `release.yml` and `rc-assets.yml` build `specsync-linux-x86_64` and `specsync-linux-aarch64` natively on GitHub's Ubuntu 22.04 runners (`ubuntu-22.04` and `ubuntu-22.04-arm`) instead of `ubuntu-latest`, so no Linux glibc download needs a glibc newer than 2.35 (INSTALL-1).
- **Acceptance**: Both release workflows build a new static `specsync-linux-aarch64-musl` download natively on `ubuntu-24.04-arm` with `musl-tools`, and the release artifact validator expects it alongside the existing five archives (INSTALL-1.b).
- **Acceptance**: `.github/scripts/linux-baseline.py check` verifies a packaged Linux archive against its `.sha256` sidecar, extracts the single binary and runs it with no network, as user 65534 on a read-only root filesystem, in digest-pinned containers: glibc downloads on `ubuntu:22.04` (the floor) and `debian:12-slim` (the system reported in #882), musl downloads on those two plus `debian:11-slim` and `alpine:3.22`.
- **Acceptance**: That check passes only when `--version` prints the package version, `--help` and `init` succeed, and `check --strict --require-coverage 100` passes on a one-module project and then fails once an undocumented export is added; any other result fails the check with a message naming the image, the step, the output and the rebuild recovery (INSTALL-1).
- **Acceptance**: `release.yml` and `rc-assets.yml` run that check on every packaged Linux archive after its checksum is written and before the upload step, so a Linux download that does not start on its baseline is never uploaded or released (INSTALL-1.c).
- **Acceptance**: A new `ci.yml` job, `linux-baseline`, builds the four Linux archives with the same runner, target and build command as `release.yml` and runs the same check on every full CI run; it is wired into `implementation-gate` and the scoped-review summary.
- **Acceptance**: On its ARM64 glibc entry that job also requires the check to reject the published, checksum-pinned v6.2.0 `specsync-linux-aarch64.tar.gz` with `GLIBC_2.39` in the output (the #882 regression).
- **Acceptance**: `.github/scripts/test-linux-baseline.py` fails when the Linux runner and target pairs differ between the check script, `ci.yml`, `release.yml` and `rc-assets.yml`, when either release workflow lacks the check before its upload step, when a baseline image is not pinned by digest, when the release artifact validator misses a Linux download, or when README or the GitHub Action docs stop naming the floor and every Linux download; it also unit-tests the check's checksum, archive and transcript handling against a fake `docker`, and CI and the `verify` lane run it.
- **Acceptance**: README and the GitHub Action docs name the floor (glibc 2.35: Ubuntu 22.04, Debian 12 and newer) (INSTALL-1.a), the static musl download for each architecture as the fallback below it (INSTALL-1.b), and the `version 'GLIBC_...' not found` start-up error as the sign that a system is below the floor.
- **Acceptance**: Failing test first: before the fix the check fails locally on the published v6.2.0 ARM64 archive and the structural test fails on the current workflows; after the fix the check passes locally on ARM64 archives built with the new recipe (glibc in an Ubuntu 22.04 container, musl in an Ubuntu 24.04 container) and the structural test passes; both observations are recorded as Chough evidence, and no tag or release is created or published.

## Evidence

- Verification commit: `ebbbfb25e4e59943a8718f63b3dadc36bec2bc79`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

spec-sync#882 records independent audit finding D6 (plan row S7). The checksum-verified 6.1.2
`specsync-linux-aarch64` exits before `--version` on Debian 12 (glibc 2.36) with
`version 'GLIBC_2.39' not found`. A source build of the same release works there.

Human intent: INSTALL-1, INSTALL-1.a, INSTALL-1.b and INSTALL-1.c in `hi/install.md`.

Human decision, quoted as relayed: Leif (2026-11-05 20:35 MST, orc tab, relayed by orc t18):
"Both arches, glibc 2.35, + musl". The relay says: fix the x86_64 and ARM64 gnu builds on
Ubuntu 22.04 (floor glibc 2.35 = Ubuntu 22.04 / Debian 12+); add `specsync-linux-aarch64-musl`
as the fallback; a release whose Linux download would not start on the floor is stopped (CI runs
the built binaries in ubuntu:22.04 and debian:12). No tag or publish. The rejected alternatives
were ARM64-only scope and `cargo install` as the only ARM64 fallback.

What a session picking this up needs to know:

- The agent verified the cause locally (see research.md). Both v6.2.0 gnu binaries, aarch64 and
  x86_64, need `GLIBC_2.39` only through two weak Rust std symbols, `pidfd_spawnp` and
  `pidfd_getpid`. They acquire that version because they are linked on `ubuntu-latest`
  (Ubuntu 24.04). The loader enforces the version even for weak symbols.
- The packaged Action downloads the gnu binary, so it also fails today on GitHub's
  `ubuntu-22.04` and `ubuntu-22.04-arm` runners.
- CI is archive-first. The new `linux-baseline` job first runs on the archived tip, so local
  container runs are the pre-review evidence for the recipe, and the CI run on the archived tip
  is the hosted confirmation. A failure there is fixed by a commit on top, keeping the scope.
- Siblings edit `CHANGELOG.md` in parallel. Merge origin/main before opening the PR and again
  before merge; never rebase or force-push.
- Chough records for this cycle live in `.chough/sessions/882-linux-baseline.json`. The shared
  `.chough/session.json` stays at main's bytes (orc instruction).

## From the change's design.md

# Design

## One table, four Linux downloads

`.github/scripts/linux-baseline.py` owns the table that every other file is held to:

| Artifact | Target | Build runner | Baseline images it must start on |
|---|---|---|---|
| `specsync-linux-x86_64` | `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | ubuntu:22.04, debian:12-slim |
| `specsync-linux-x86_64-musl` | `x86_64-unknown-linux-musl` | `ubuntu-latest` (static, unchanged) | ubuntu:22.04, debian:12-slim, debian:11-slim, alpine:3.22 |
| `specsync-linux-aarch64` | `aarch64-unknown-linux-gnu` | `ubuntu-22.04-arm` (native; the cross linker goes away) | ubuntu:22.04, debian:12-slim |
| `specsync-linux-aarch64-musl` (new) | `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` (native, `musl-tools`) | ubuntu:22.04, debian:12-slim, debian:11-slim, alpine:3.22 |

The floor is glibc 2.35 (Ubuntu 22.04, Debian 12 and newer). It holds because the glibc
downloads are linked against glibc 2.35. Running them on the floor image is the authoritative
proof, because the dynamic loader checks every version requirement, weak symbols included. A
separate symbol scan would add nothing.

## The check

`linux-baseline.py check --archive <dir>/<artifact>.tar.gz --expect-version <version>`:

1. **Artifact and host.** The artifact name comes from the archive file name and must be in the
   table. The host architecture must match the artifact's. Otherwise it refuses and names the
   recovery: run on a runner of that architecture, as the `linux-baseline` CI job does.
2. **Checksum.** `<archive>.sha256` must be exactly `<64 hex>  <archive name>\n` and match the
   archive's SHA-256. This is the same record the release workflow writes and verifies.
3. **Archive.** The archive must hold exactly one regular file named after the artifact. Links,
   directories, extra members and any other name are refused. The bytes are written to a fresh
   temp directory, mode 0755.
4. **Per image.** `docker pull` by digest (three attempts, because Docker Hub can be flaky), then
   `docker run --rm --pull never --network none --read-only --tmpfs /tmp --cap-drop ALL
   --security-opt no-new-privileges --user 65534:65534` with the binary, a one-module fixture
   and a POSIX `sh` smoke script, all mounted read-only. The script runs five steps and prints a
   `@@step <name> <exit>` marker, then that step's output prefixed `@@out `:
   1. `--version`
   2. `--help`
   3. `init`, after which the fixture is copied in
   4. `check --strict --require-coverage 100`
   5. the same check after appending `pub fn goodbye() {}`
5. **Verdict.** Each image passes only if:
   - `version` exits 0 and its first line is `specsync <expected version>`;
   - `help` exits 0 and prints `Usage:`;
   - `init` and `check` exit 0;
   - `drift` exits non-zero and names `goodbye`.

   Every failure names the artifact, the image and its glibc, the step, the exit and the first
   output lines. A loader failure (`GLIBC_` and `not found`) adds the recovery: build it with
   `cargo build --release --target <target>` on the `<runner>` runner, as `release.yml` does, or
   raise the documented floor in the docs and this table together.
6. **Report.** A JSON summary goes to stdout: artifact, sha256, and per image the steps with their
   exits. The exit is 0 only when every image passed.

## Workflows

- `release.yml` `build` and `rc-assets.yml` `build`:
  - Matrix runners and targets follow the table, plus the new aarch64 musl row.
  - The cross-linker step is removed. The musl toolchain step sets the linker and `CC_*` per target.
  - A step "Start the packaged Linux archive on its documented baseline" runs the check (`if:
    runner.os == 'Linux'`). It sits after the archive and its checksum are written and before the
    upload step. In `release.yml` the version comes from `needs.resolve.outputs.version`; in
    `rc-assets.yml` it comes from the checked-out `Cargo.toml` (`cargo pkgid`).
  - A comment states the floor, and what to do when GitHub retires its Ubuntu 22.04 images: move
    the build into a digest-pinned `ubuntu:22.04` container, or raise the floor deliberately in
    the docs and the table together. Never let `ubuntu-latest` raise it silently.
- `ci.yml` `linux-baseline` (`needs: classify`, selected on `full`, matrix from the same table):
  - Checkout, the toolchain pin, `musl-tools` for musl targets, and a per-target rust cache.
  - `cargo build --release --target <target>`, then package and checksum exactly as the release
    does, then the check.
  - On `specsync-linux-aarch64` only, the negative control: download the published v6.2.0 archive,
    write a sidecar with the pinned SHA-256
    `86e081a5ef8084f1282d77d807e495b1dbdbc8ae89840594ed3e915439bcf35a`, and require the check to
    exit non-zero with `GLIBC_2.39` in its output.
  - Added to `implementation-gate` `needs` and its GATES row, and to the `corvid-pet` summary.
  - `ci.yml` path filters gain `.github/workflows/rc-assets.yml`, which is now held to the table.
- `validate-release-candidate.py`: `EXPECTED_ARTIFACT_ARCHIVES` gains `specsync-linux-aarch64-musl`.
  Its test fixtures follow.

## Structural test

`test-linux-baseline.py` (unittest) reads the workflows through Ruby's standard Psych parser, the
way the existing workflow validators do. It asserts that:

- the Linux build rows of `release.yml`, `rc-assets.yml` and `ci.yml` equal the table, by
  artifact, target and runner;
- both release workflows run the check, limited to Linux, after packaging and before upload;
- `ci.yml` runs the check and the pinned negative control, and selects the job on `full`;
- every baseline image is pinned by a 64-hex digest;
- `EXPECTED_ARTIFACT_ARCHIVES` contains every Linux artifact;
- README and the Action docs name the floor and every Linux artifact.

It also runs `check` end to end against a fake `docker` executable that records its arguments and
replays transcripts. Those cases cover pass, loader failure with its recovery text, wrong version,
drift not detected, a bad checksum, a CRLF sidecar, a tar with an extra member or a link, an
unknown artifact, and an architecture mismatch. It asserts that the isolation flags reach
`docker run`.

## From the change's testing.md

# Testing

| What | Where | Proves |
|---|---|---|
| Check on published v6.2.0 aarch64 archive (red) | local Docker, arm64 | the check catches #882 on ubuntu:22.04 and debian:12 |
| Structural test on the unmodified workflows (red) | local | the test notices the defect class |
| Check on ARM64 glibc archive built with the new recipe in `ubuntu:22.04` | local Docker, arm64 | the glibc fix starts on the floor and the reported system |
| Check on ARM64 musl archive built in `ubuntu:24.04` with `musl-tools` | local Docker, arm64 | the fallback starts on all four images, including Debian 11 and Alpine |
| `test-linux-baseline.py` (structural tests and fake-docker unit tests) | local, CI `validate-action`, `verify` lane | workflow, table and doc lockstep; check logic and refusals |
| `test-validate-release-candidate.py` | local, CI | release artifact set includes aarch64 musl |
| `test-required-ci-gate.py` | local, CI | the new job is gated |
| `fledge lanes run verify`, `specsync check --strict`, `hi check`, pre-push gate | local | nothing else regressed |
| `linux-baseline` CI job: four archives plus the negative control | GitHub, archived tip | the hosted recipe on the real runners |

x86_64 archives cannot run natively on this arm64 host. Their first native run is the CI job.

Results and commands are recorded below and as Chough evidence while the work runs.

## Red, before the fix (2026-11-05, base `ad50bdef`, local macOS arm64, Docker 29.8.1)

1. **Published v6.2.0 ARM64 archive.** The archive is
   `specsync-linux-aarch64.tar.gz`, sha256
   `86e081a5ef8084f1282d77d807e495b1dbdbc8ae89840594ed3e915439bcf35a`, with its sidecar verified.
   The command was `python3 -I .github/scripts/linux-baseline.py check --archive
   specsync-linux-aarch64.tar.gz --expect-version 6.2.0`. It exited **1**. On both `ubuntu:22.04`
   and `debian:12-slim`, `--version` exited 1 with:

   ```text
   /opt/specsync/specsync: /lib/aarch64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by /opt/specsync/specsync)
   ```

   Each failure message names the recovery: `cargo build --release --target
   aarch64-unknown-linux-gnu` on the ubuntu-22.04-arm runner.
2. **Positive control, run in the same session so the red is not a broken check.** Both ARM64
   archives were built from this source with the new recipe and packaged the way the release
   packages them (one member, LF sha256 sidecar):
   - glibc: built in `ubuntu:22.04`, Rust 1.89.0, `cargo build --release --locked`; the check
     exited **0** on `ubuntu:22.04` and `debian:12-slim`;
   - musl: built in `ubuntu:24.04` with `musl-tools`; the check exited **0** on `ubuntu:22.04`,
     `debian:12-slim`, `debian:11-slim` and `alpine:3.22`.

   On every image, `version`, `help`, `init` and `check` exited 0 and `drift` exited 1. The
   check also refused the first local packaging attempt because bsdtar added AppleDouble `._`
   members (exit 2, "must hold exactly …"), which is the intended strictness.
3. **Structural test on the unmodified tree.** `python3 .github/scripts/test-linux-baseline.py`
   exited **1** with 15 failures, and the 23 check-logic unit tests passed. The failures cover:
   - the gnu rows on `ubuntu-latest` and no aarch64 musl row, in both release workflows;
   - no check step before upload;
   - the aarch64 cross linker still present;
   - no `linux-baseline` CI job, gate wiring or negative control;
   - `rc-assets.yml` missing from the CI path filters;
   - the release validator lacking `specsync-linux-aarch64-musl`;
   - README and the Action docs not naming the floor.

Chough evidence for these runs is recorded after approval, by re-running them before the fix,
because the evidence links REQ-github-025, which `change check` materializes only after approval.

## Green, after the fix (2026-11-05, merged tree `4547d9c0`, local macOS arm64, Docker 29.8.1)

The same three Chough cases that failed before the fix now pass. The bug is
`verified` (reproduced, then passed later).

- **`TC-github-linux-arm64-glibc-download-starts-on-baseline`**: passed
  (`OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453`).
  - Archive: `specsync-linux-aarch64.tar.gz`, sha256
    `62ffb9aaf5a32425d00c183074c371eb4ea5f24a3bfebc2236d2811cf565533b`.
  - Built natively in `ubuntu:22.04` (glibc 2.35, Rust 1.89.0 via rustup) with
    `cargo build --release --target aarch64-unknown-linux-gnu`, then packaged with GNU tar and
    `sha256sum` as the Package step does.
  - Ran on `ubuntu:22.04` and `debian:12-slim`.
- **`TC-github-linux-arm64-musl-download-starts-anywhere`**: passed
  (`OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b`).
  - Archive: `specsync-linux-aarch64-musl.tar.gz`, sha256
    `121bc45462a1fef046668866df9fd7c2e4e74512b8658c7a70600b78d57142f0`.
  - Built in `ubuntu:24.04` with `musl-tools` and `CC_aarch64_unknown_linux_musl=musl-gcc`.
  - Ran on `ubuntu:22.04`, `debian:12-slim`, `debian:11-slim` and `alpine:3.22`.
- **`TC-github-linux-baseline-lockstep`**: passed
  (`OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991`), 34 tests.
- **`fledge lanes run verify`** with `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.89.0, per
  `rust-toolchain.toml`): passed, all 7 steps in 35m23s.
  - fmt, clippy `-D warnings` and `cargo check`;
  - `cargo test`: 2823 + 483 passed, 2 ignored as on main, 0 failed;
  - release build;
  - `spec-check` 62/62 at 100% coverage;
  - `release-candidate-test`, `ci-gate-test` and `linux-baseline-test`.
- Also passed: `validate-workflow-runtime-pins.py`, `validate-release-version.py`,
  `test-classify-ci-paths.sh`, `test-archive-readiness.sh`, `hi check`, and the site build
  (43 pages).

Not covered locally:
- The x86_64 archives cannot run natively on this arm64 host.
- The hosted runners (`ubuntu-22.04`, `ubuntu-22.04-arm`, `ubuntu-24.04-arm`) and the v6.2.0
  negative control in CI first run as the `linux-baseline` job on the archived tip.

## Where these lessons go

- `specs/github/github.3md`
