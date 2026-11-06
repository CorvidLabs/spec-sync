---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: docs
---

# Docs

- `docs/RELEASING.md`: the release-candidate lane runs the suite with a pinned cargo-nextest.
- `CHANGELOG.md` Unreleased: fewer git processes per lifecycle command; release-candidate suite under nextest.
- Spec Change Log rows for git_utils, change and cmd_change.
