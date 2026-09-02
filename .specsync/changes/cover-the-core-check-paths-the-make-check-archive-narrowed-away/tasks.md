---
change: cover-the-core-check-paths-the-make-check-archive-narrowed-away
artifact: tasks
---

# Tasks

- [x] Confirm the four paths were changed by the core-check commits (84cb5cae, 359eeee2) and
      are absent from both archived records' `affected_paths` on this branch
- [x] Confirm `git diff --name-only <merge-base>..HEAD` lists no other meaningful path that
      neither archived record covers (the audit names exactly these four)
- [x] Create the record naming `site/src/content/docs/deltas.md`, `src/commands/check.rs`,
      `src/commands/init.rs`, `tests/integration/commands.rs` with `--no-spec-change`
- [x] Declare `cmd_check` and `cmd_init`, the canonical owners of the two production files, so
      the scoped check validates them

The lifecycle steps that follow (check, review, finalize, archive tip) and the two gates the
record exists for (`change audit --strict` exits 0; `check --strict` still passes) are the
acceptance criteria in `change.md`, not tasks of this record.
