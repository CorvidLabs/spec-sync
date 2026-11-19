---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: docs
---

# Docs

- `site/src/content/docs/cli.md`: `change approve --dry-run` beside `change approve`: what it checks,
  that it writes nothing, the text lines, the JSON document with its check names, and the exit codes.
- `CHANGELOG.md`, Unreleased, Added: the dry run, for people and for tools that call it.
- `AGENTS.md`: one row in the command table.
- `specs/change/change.3md`: one lesson plane, numbered when the branch is last updated against main.
- The three specs: Public API rows for the new public items and `files:` entries for the new files.
