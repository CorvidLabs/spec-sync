---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: tasks
---

# Tasks

1. `conflict_hunks`, `conflict_free_side`, `unmerged_paths` (Option), and a
   memo keyed by repo root.
2. `ExportScan::Conflicted` carrying per-side evidence; produce it from the one
   internal scan every entry point funnels through.
3. Refuse in `validate_spec` for both source and body; blank fenced code first.
4. Cover the `issues` read path.
5. Prove the guard does not fire on this repository.
