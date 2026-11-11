---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: tasks
---

# Tasks

- [x] Split unowned inputs into modified and untouched since the change's base commit
- [x] Narrowing recovery for untouched inputs; declare recovery only for modified ones
- [x] Unit test with one modified and one untouched unowned file under one directory scope
- [x] REQ-cmd-change-008 and REQ-change-053 name the narrowing recovery
- [x] Review round (corvid-agent at 0ac9b11a): NUL-separated git output so non-ASCII paths match; fail-first non-ASCII test
- [x] Review round (independent, at 3aadc6f7): base_commit validated and passed after --end-of-options (option injection); one send-back for the mixed case; blocker hints agree; merge-base and --no-renames in the changed set; redundant narrowing dropped; quoted '<path>,<path>' placeholder; fallback stated in both deltas
