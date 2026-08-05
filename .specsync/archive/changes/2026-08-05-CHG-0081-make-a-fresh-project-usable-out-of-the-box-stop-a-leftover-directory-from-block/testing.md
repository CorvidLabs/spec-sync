---
change: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-050` | `cargo test` 2,181 unit + 333 integration passed; sandbox drill 031-specsync-merge-conflict.sh (7/7) asserts change new succeeds on a branch missing an earlier change directory and that a husk is not listed as active; drill 032-next-action-loop.sh independently reproduces the init empty-command condition. |

## Manual verification

- Fresh project with no build system: init warns, naming the file and an example.
- Fresh project with package.json: init detects `npm test` and does not warn.
- Branch A creates a change and commits; branch B off main runs change new and succeeds.
- `audit --strict` passes on a complete draft, where it previously reported the
  sequence ledger as uncovered.

## Not covered

Two independent clones can still allocate the same ordinal. Unchanged by this work.
