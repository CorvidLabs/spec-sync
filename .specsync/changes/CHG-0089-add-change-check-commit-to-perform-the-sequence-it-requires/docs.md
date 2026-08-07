---
change: CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires
artifact: docs
---

# Docs

Surface in CLI help for `change check`:

- `--commit` — verify, commit materialization, re-verify, commit evidence
- `--push` — after `--commit`, push (requires `--commit`)

Agent pack / AGENTS.md should prefer `change check --commit` when the working
tree must match CI-accepted verification evidence.
