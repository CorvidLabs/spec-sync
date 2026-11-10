---
change: keep-a-clean-merge-driver-result-faithful-to-both-sides-quote-yaml-typed-list-items-end-the-change-log-at-its-first
artifact: docs
---

# Docs

- `CHANGELOG.md` Unreleased, Fixed: the git merge driver no longer reports a clean merge that changes what a side wrote. Quoted YAML-typed list items stay strings; text after the change log stays in place; version comments merge three ways in either order; renumbering stops at `u64::MAX` (#870, #872, #875, #876).
- `specs/merge/merge.spec.md`: invariants 22 and 23 amended, and a Change Log row.
- `specs/merge/merge.3md`: a lesson plane (a clean report is a claim about the result, not only about the merge).
- `specs/merge/tasks.md`: the four issues, checked off.
- `hi/merge.md`: MERGE-5 and its cases, with their source. `INTENT.md` index regenerated.
