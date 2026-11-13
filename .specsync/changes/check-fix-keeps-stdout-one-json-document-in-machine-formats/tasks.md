---
change: check-fix-keeps-stdout-one-json-document-in-machine-formats
artifact: tasks
---

# Tasks

- [x] Red-first integration tests: `--fix --json` and `--fix --dry-run --json` stdout parse as one JSON document, with the per-spec line on stderr
- [x] Print auto_fix_specs' per-spec lines on stderr when the format is not text
- [ ] Targeted fix tests, cmd_check unit tests, fmt and clippy
