---
change: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
artifact: tasks
---

# Tasks

- [x] Replace unconditional duplicate stage-zero rejection with deterministic pair accumulation.
- [x] Preserve identical duplicate entries and reject differing mode or object pairs without
      mutating the first observation.
- [x] Add the cross-batch parent/child characterization regression.
- [x] Add independent conflicting-mode and conflicting-object fail-closed regressions.
- [x] Run the focused tests, repository verification lane, strict spec check, and trust gate.
