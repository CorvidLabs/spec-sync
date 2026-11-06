---
change: build-the-linux-glibc-release-binaries-on-ubuntu-22-04-add-a-static-aarch64-musl-fallback-and-stop-a-release-whose
artifact: requirements
---

# Requirements

- REQ-github-024 (added, `github`): the prebuilt Linux release downloads start on their documented
  baseline. The glibc downloads are linked on Ubuntu 22.04, so they start on glibc 2.35 or newer
  (Ubuntu 22.04, Debian 12 and newer). The static musl downloads, one per architecture, are the
  fallback below that floor. Both release workflows refuse to upload a Linux archive that fails
  `linux-baseline.py check`, and PR CI runs the same recipe and check. See `deltas/github.md`.
- Traces to HI:
  - INSTALL-1: the download starts on the oldest Linux the guide supports;
  - INSTALL-1.a: the floor is named;
  - INSTALL-1.b: the musl fallback;
  - INSTALL-1.c: a release that would not start is stopped.
- No `REQ-change` number is used.
