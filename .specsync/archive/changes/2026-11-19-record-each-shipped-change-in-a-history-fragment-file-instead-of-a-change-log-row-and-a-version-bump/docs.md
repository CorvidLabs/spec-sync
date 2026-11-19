---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: docs
---

# Docs

- `site/src/content/docs/spec-format.md` and `workflow.md`: where a change's history lives, that `change check` no longer bumps `version:` or adds a row, that existing tables stay as they are, and `specsync history`.
- `site/src/content/docs/cli.md`: the `history` command.
- `site/src/content/docs/deltas.md`: the Change Log note no longer says check appends a row.
- `AGENTS.md`: `specsync history` in the quick reference.
- `CHANGELOG.md` `[Unreleased]`: the entry, including that every machine running change commands in a repository with fragments needs this release.
