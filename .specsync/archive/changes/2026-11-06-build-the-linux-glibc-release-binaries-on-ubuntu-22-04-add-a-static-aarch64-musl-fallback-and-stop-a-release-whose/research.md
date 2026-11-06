---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: research
---

# Research

All findings were run locally on 2026-11-05 on a macOS arm64 Docker host. The images are pinned
by multi-arch index digest; each index carries both linux/amd64 and linux/arm64.

| Image | Digest | glibc |
|---|---|---|
| `ubuntu:22.04` | `sha256:076e7bf82319619f28dd0aa9c02ab34b055ad37100395b4f49a2ffba04fb5e11` | 2.35 |
| `debian:12-slim` | `sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc` | 2.36 |
| `debian:11-slim` | `sha256:e5b6442dd2e9684cf5e87d8338b5968f3b348636fc0be6d7850a381e3731a2bd` | 2.31 |
| `alpine:3.22` | `sha256:bfe7ceb9f09ec0566c48fb93356e45a47a00be5a1dd14fa5b909e7709c66b034` | none (musl) |

1. **Published v6.2.0, checksums verified against their sidecars.** `objdump -T` on both
   `specsync-linux-aarch64` and `specsync-linux-x86_64` shows 12 symbols at `GLIBC_2.34` as the
   highest strong requirement. Two weak undefined symbols carry `GLIBC_2.39`:
   `pidfd_spawnp` and `pidfd_getpid`.
2. **v6.2.0 aarch64 fails to start** with `version 'GLIBC_2.39' not found` on `ubuntu:22.04` and
   on `debian:12-slim`, as the audit found for 6.1.2 on Debian 12. v6.2.0 x86_64 fails the same
   way on `ubuntu:22.04` under linux/amd64 emulation.
3. **Same source built on Ubuntu 22.04.** `cargo build --release --locked` with Rust 1.89.0 in
   `ubuntu:22.04` arm64 gives a binary whose highest requirement is `GLIBC_2.34`. Run offline, as
   user 65534, on a read-only rootfs with no git, on both `ubuntu:22.04` and `debian:12-slim`:
   - `--version` printed `specsync 6.2.0`;
   - `--help` and `init` exited 0;
   - `check --strict --require-coverage 100` passed on a one-module project;
   - after an undocumented export was added, the check exited 1 naming it.
   On `debian:11-slim` it fails with `GLIBC_2.34' not found`, which is below the floor, as
   expected.
4. **Static `aarch64-unknown-linux-musl`**, built natively in `ubuntu:24.04` arm64 with
   `musl-tools` and `CC_aarch64_unknown_linux_musl=musl-gcc`. It is a statically linked, stripped
   ELF and passed the same run on `debian:11-slim` and `alpine:3.22`. The tree-sitter grammars
   need `musl-gcc` for their C code, as #392 found for x86_64.
5. **GitHub's Ubuntu 22.04 arm64 runner image lists Docker** (actions/partner-runner-images
   `arm-ubuntu-22-image.md`), so the check can run on the runner that builds the binary.
