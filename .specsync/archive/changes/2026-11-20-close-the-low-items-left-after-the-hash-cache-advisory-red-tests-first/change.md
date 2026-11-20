---
id: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
state: archived
type: bug_fix
base_commit: e334bd8abd8f5a2343040d45ca1308cad9e33085
---

# Close the low items left after the hash-cache advisory, red tests first

## Intent

Close the low items left after the hash-cache advisory, red tests first

## Affected Canonical Specs

- `util`
- `hooks`
- `hash_cache`
- `change`
- `commands`
- `cmd_resolve`
- `cmd_stale`
- `cmd_change`
- `cmd_scaffold`
- `cmd_check`
- `validator`

## Acceptance Criteria

- Each red test committed in e334bd8a passes, and each failed on 6.4.0: W1, a linked .specsync/archive/changes is refused naming the folder and no outside name is printed; S1, no hook is written into an existing outside folder the repository's own config names; S2, a journal that does not record the project now at its path is never replayed, and lifecycle commands stop with its path; S3, correct-owner --manifest and scaffold --template refuse a FIFO or an oversized file instead of waiting, still following links; S5, the journal is kept in the user's state folder, not the cache folder; L2, a state folder the current user does not own or that others can write is refused; X1, the scans flag read_dir, WalkDir::new and Git's apply, merge, switch and config unless marked; X2, a hooks path spelled through a link to the work tree is inside it; N2, a state folder that would lie inside the project is refused; H1, a linked worktree's shared hooks folder is made when missing; C1, the change read cache is read and written only through the local state handles; C2, check --only-status keeps a refused spec with its refusal; C3, resolve reports a refused spec; C4, stale counts a refused spec as unmeasurable; C5, a hash cache over the bound is refused, not written. The five history clauses in REQ-util-006, REQ-util-008, REQ-change-162, REQ-hooks-009 and REQ-cmd-check-021 are removed with no change of meaning. macOS verify lane and Linux are green, and change audit passes.

## No-spec Rationale

Not applicable
