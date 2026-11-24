---
change: keep-archived-coverage-recoverable-and-visible
artifact: research
---

# Research

The reproductions under `target/specsync-handoff/qualification-2026-11-23` are `specsync-1132-reopen-cli.log` and `specsync-1132-malformed.log`. The former checks a genuine signed archive through reopen, check, review, and refused finalization. The latter checks both CLI formats against valid, absent, truncated, wrong-ID and oversized controls. The current bounded reader and `write_prepared_files` journal provide the seams for the change.
