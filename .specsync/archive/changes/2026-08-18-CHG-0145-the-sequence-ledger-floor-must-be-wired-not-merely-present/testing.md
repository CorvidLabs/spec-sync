---
change: CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-013 | `git_commit_all_raises_a_stale_ledger_before_staging_it`, shown below to fail when the raise is removed from the staging path |

## Discrimination

Run against a copy of the tree with the floor call physically removed from
`git_commit_all`, verified by grep that the call site was gone rather than
assuming the edit applied:

| tree | result |
|---|---|
| floor call present | ok |
| floor call deleted | **FAILED** — "committing 1 over a committed 3 is the #533 regression" |

Before this test existed, that same deletion left `cargo test` entirely green.

## Suite

`cargo test` rc=0 — 2299 unit, 405 integration, 0 failures.
`cargo clippy -- -D warnings` rc=0. `cargo fmt --check` rc=0.
