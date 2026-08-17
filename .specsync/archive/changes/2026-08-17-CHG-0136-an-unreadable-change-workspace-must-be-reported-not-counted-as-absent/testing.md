---
change: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-change-011 | Gate 055 goes `pass=5 pending=2` to `pass=7 pending=0` on a fixed binary and fails on an `origin/main` binary built from a separate checkout. The two pending assertions are the empty-project line over a corrupt sibling in `list` and in `status`; the five controls that stay green are what prove the fix did not simply start refusing every tree. Exit status and JSON shape are covered by the three-case table below, including the genuinely-empty project that must still print the empty-project line at rc=0 |
| REQ-change-068 | `unreadable_workspace_is_reported_beside_its_healthy_siblings` asserts both halves separately — the healthy record survives, so enumeration no longer aborts, and the bad one is named with its path, so the failure is no longer discarded — then removes the corruption and asserts a clean complete roster returns. That tail is the vacuity control: without it, a change reporting every workspace unreadable would satisfy every assertion above it. The fail-closed adapter for the eleven internal digest and ledger callers is covered by the unchanged suite, 2277 unit tests passing |

## Suite

    cargo test                    rc=0    2277 unit passed, 374 integration passed, 0 failed
    cargo clippy -- -D warnings   rc=0
    cargo fmt --check             rc=0

309 → 310 `#[test]` markers in `src/change_tests.rs`, counted before and after rather than read
off a diff: exactly one test added.

`cargo clippy --all-targets` is red, but it is red on unmodified `main` with a byte-identical
finding set. Pre-existing debt, filed as #608 (test code has never been lint-gated because CI
omits `--all-targets`). Not introduced here.

## Behavioural verification, three cases

| case | before | after |
|---|---|---|
| two changes, one corrupt | `rc=0  No active SDD changes.` | `rc=1`, healthy change listed, corrupt one named with path and `line:column` |
| record downgraded by a 5.2 write (#603) | `rc=0  No active SDD changes.` | `rc=1`, `workflow-v1 change CHG-… was not present at the trusted pre-v2 cutoff f7f7f3e6…` |
| **genuinely empty project** | `rc=0  No active SDD changes.` | **`rc=0  No active SDD changes.`** |

The third row is the vacuity control and is the reason the other two mean anything: a change that
simply started refusing every tree would satisfy rows one and two and fail row three.

## JSON, both shapes, one document each

    healthy   rc=0   bare array, byte-identical to before
    degraded  rc=1   object: changes=1, unreadable names the corrupt id, error present

Both parse. The first attempt did not: `cmd_change`'s tail handler prints its own `{"error": …}`
in JSON mode, so printing a roster *and* returning `Err` emitted two concatenated documents and
made stdout unparseable. Caught by parsing the output rather than eyeballing it.

## Unit regression, with its own control

`unreadable_workspace_is_reported_beside_its_healthy_siblings` asserts both halves — the healthy
record survives (enumeration no longer aborts) and the bad one is named (the failure is no longer
discarded) — then removes the corruption and asserts a clean, complete roster returns. Without
that tail, a change that reported every workspace unreadable would pass every assertion above it.

## Sandbox gate 055, discriminated on two genuinely different binaries

    UNFIXED  c977572e   rc=1   pass=5  pending=2  fail=0
    FIXED               rc=0   pass=7  pending=0  fail=0

Both pending gates converted; all five controls stayed green. The controls are what prove the fix
did not simply start refusing everything: two healthy changes still list as two, `show` on the
healthy id still works, `show` on the corrupt id still names the file, and deleting the corrupt
workspace restores the listing.

The unfixed binary was built from a separate checkout of `origin/main`, not by reverting files in
the working tree. That method was adopted after a reverted-file build failed to compile earlier in
this effort, leaving the drills running against the still-fixed binary and passing — a false
discrimination proof caught only by checking the build's exit code.

## Whole-board check

    binary:  wt443 release, verified to carry both #539 and #443 by string inspection
    pass=44  fail=11  skip=0  total=55

Against the `c977572e` baseline of `42/13`, exactly two drills changed state — 048 (#539, from
the merge) and 055 (this change). Nothing else moved. The eleven remaining failures are unchanged
and each pins a bug still open.

Binary identity was confirmed before the run rather than inferred from a timestamp. Two earlier
board runs in this effort were invalid for exactly that reason: one used a 5.2 binary found on
`PATH`, the other a pre-rebase artifact.
