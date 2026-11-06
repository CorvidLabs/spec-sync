---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: design
---

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
