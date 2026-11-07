---
change: cover-the-two-paths-900-changed-outside-its-declared-scope
artifact: research
---

# Research

- `change audit --strict` compares the paths a pull request changes with the affected paths of its changes and refuses any that are not covered (`uncovered_paths_error`). Its remediation names `change new ... --path` with `--no-spec-change` when no spec text changes.
- `git diff origin/main...HEAD` limited to the two files shows exactly #900's edits: +2/-1 in `correction_prefix_digest` and +14/-1 in the key-set test. Nothing else in the pull request is uncovered.
- The local verify lane does not run `change audit`, and CI never reached that gate before archival. From now on, `specsync change audit --strict` runs before every push.
