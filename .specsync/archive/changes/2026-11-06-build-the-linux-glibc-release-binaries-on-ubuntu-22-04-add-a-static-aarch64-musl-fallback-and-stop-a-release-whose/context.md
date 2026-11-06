---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: context
---

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
