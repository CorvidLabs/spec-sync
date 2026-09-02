---
change: cover-the-core-check-paths-the-make-check-archive-narrowed-away
artifact: plan
---

# Plan

Declare the four already-committed paths and carry the record through the lifecycle with no
code or spec edits:

1. `change new` with `--no-spec-change`, `--spec cmd_check --spec cmd_init` (the canonical
   owners of the two production files) and one `--path` per file.
2. Approve, then `change check --commit`: the scoped check validates `cmd_check` and
   `cmd_init` against the tree; no spec text is materialized.
3. Independent scoped review, `change review --verdict pass`, `change finalize`, commit the
   archive tip.
4. Confirm `specsync change audit --strict` exits 0 on the tip and `specsync check --strict`
   still passes.
