## ADDED

### REQUIREMENT REQ-github-024

The prebuilt Linux release downloads SHALL start on their documented baseline.

- The glibc downloads, `specsync-linux-x86_64` and `specsync-linux-aarch64`, SHALL be linked against glibc 2.35 by building them natively on GitHub's Ubuntu 22.04 runners. They then start on glibc 2.35 or newer: Ubuntu 22.04, Debian 12 and newer.
- The static musl downloads, `specsync-linux-x86_64-musl` and `specsync-linux-aarch64-musl`, SHALL start on any Linux of their architecture. They are the documented fallback below that floor.
- `release.yml` and `rc-assets.yml` SHALL run `.github/scripts/linux-baseline.py check` on every packaged Linux archive before upload, and SHALL NOT upload an archive that fails it.
- Pull-request CI SHALL build the same archives with the same runner, target and build command, and SHALL run the same check.

#### Acceptance Criteria

- The check verifies the archive against its `.sha256` sidecar. It runs the extracted binary with no network, as user 65534, on a read-only root filesystem, in digest-pinned containers: glibc downloads on `ubuntu:22.04` and `debian:12-slim`; musl downloads on those two plus `debian:11-slim` and `alpine:3.22`.
- An image passes only when `--version` prints the package version, `--help` and `init` succeed, `check --strict --require-coverage 100` passes on a one-module project, and the same check fails once an undocumented export is added.
- A failure names the artifact, the image, the step, the exit code and the output. A loader `GLIBC_… not found` failure also names the rebuild on the floor runner.
- The check rejects the published v6.2.0 `specsync-linux-aarch64.tar.gz`, which needs `GLIBC_2.39`.
- The Linux runner and target pairs in `ci.yml`, `release.yml` and `rc-assets.yml` equal the check script's table, and a test fails when they drift.
- README and the GitHub Action docs name the floor, the musl fallback for each architecture, and the `GLIBC_… not found` start-up error.
- The release artifact validator expects six archives, including `specsync-linux-aarch64-musl`.
