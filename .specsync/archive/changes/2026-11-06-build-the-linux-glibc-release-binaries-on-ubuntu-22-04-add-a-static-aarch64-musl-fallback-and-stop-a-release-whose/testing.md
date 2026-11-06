---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: testing
---

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
