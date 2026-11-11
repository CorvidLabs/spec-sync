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
- [x] Second-review round: mixed recovery keeps the files: repair; untouched hints name their owner; untouched inputs count as covered only when a declare answer is printed; narrow-only test; refreshed design and context
- [x] Third-review round: no merge base (no remote default) or a truncated listing classifies nothing; hint grammar; REQ-change-053 bullet scoped to the preflight and review recovery; real-repo test of the changed set
- [x] Fourth-review round: the edited test file is in affected_paths (send-back, re-approval); narrowing wording names what must leave the scope; refreshed context and testing notes
