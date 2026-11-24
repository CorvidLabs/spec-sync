---
change: check-fix-keeps-stdout-one-json-document-in-machine-formats
artifact: tasks
---

# Tasks

- [x] Red-first integration tests: `--fix --json` and `--fix --dry-run --json` stdout parse as one JSON document, with the per-spec line on stderr
- [x] Print auto_fix_specs' per-spec lines on stderr when the format is not text
- [x] Targeted fix:: and check:: integration tests (check.rs has no unit tests), fmt and clippy
