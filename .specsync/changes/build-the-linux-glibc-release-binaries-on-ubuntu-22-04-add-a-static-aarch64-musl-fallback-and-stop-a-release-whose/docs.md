---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: docs
---

# Docs

- `README.md`, "Pre-built binaries": the floor, the musl fallback per architecture, and the
  `GLIBC_… not found` symptom with its recovery.
- `site/src/content/docs/integrations/github-action.md`, "Available Binaries": a "Runs on" column,
  the new aarch64 musl row, and the same floor and symptom note.
- `docs/RELEASING.md`: the six archives, the Ubuntu 22.04 build runners, the baseline check before
  upload, the runner-retirement recovery, and a note that the Homebrew tap does not use musl.
- `docs/HLD.md`: the release build diagram goes from 5 to 6 targets.
- `SCOPE.md`: the Linux binary line names the floor and the musl fallbacks.
- `CHANGELOG.md` Unreleased: the Linux downloads start on glibc 2.35 (Ubuntu 22.04, Debian 12) and
  the new ARM64 musl download (#882).
- `hi/install.md` and the `INTENT.md` index: INSTALL-1 to INSTALL-1.c, as captured.
- `specs/github`: REQ-github-024 in `requirements.md` (materialized), a version bump and Change Log row in `github.spec.md`, and a new lesson plane in `github.3md`.
