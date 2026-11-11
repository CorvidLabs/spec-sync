---
change: audit-reports-requirements-traversal-errors
artifact: context
---

# Context

Issue #873 (follow-up from #865's review): `duplicate_requirement_id_errors`
walks the specs tree with `WalkDir(...).filter_map(Result::ok)`, so a
directory it cannot traverse is dropped silently and `change audit` passes
without inspecting the requirements.md files below it. Unreadable *files* are
already reported; unreadable *directories* were the gap. The fix turns each
traversal error into an audit error naming the path.
