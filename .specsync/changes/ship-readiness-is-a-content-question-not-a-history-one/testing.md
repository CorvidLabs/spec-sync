---
change: ship-readiness-is-a-content-question-not-a-history-one
artifact: testing
---

# Testing

## The discriminator took three attempts, and the first two were wrong

Recorded because the failures are the useful part.

**Attempt 1** — asserted `verification_is_current` ignores a squash. **Passes on the baseline**, so
it discriminates nothing: that function was ALWAYS content-only. Nothing about it was broken. It
survives, relabelled honestly as a CHARACTERIZATION test, because it pins the property the fix
relies on.

**Attempt 2** — asserted currency before and after mutating the tree, in one process. Failed for a
reason that is not a defect: `project_input_digest` memoizes into a thread-local read scope, so a
single process cannot observe the digest move. The CLI has no such problem — each invocation is a
new process. **A control that measures a cache is not a control.**

**Attempt 3** — asks the question at the level the behaviour actually changed: `ship_status_report`.

## The discriminator

`ship_status_is_ready_after_a_squash_that_preserves_content`. Drives a change to reviewed, squashes
the branch onto main, and asserts `ready_to_finalize` with no blockers.

Verified against a separate checkout at `db36230a`:

    assertion `left == right` failed: content is unchanged, so the change is finalizable
    "ready_to_finalize": false
    "blockers": ["verification commit is not an ancestor of HEAD; re-run change check --commit ..."]
    "verification_ancestor_of_head": false

Content unchanged, review recorded, everything correct — unfinalizable purely because a squash
rewrote a hash. That is #689 in one assertion.

## Controls

**`recorded_verification_is_stale_when_the_workspace_digest_does_not_match`** — behaviour only,
passes on BOTH binaries. A fix that simply stopped checking currency would pass the discriminator
and fail this.

**Live, on a real squash fixture:**

    squash, content unchanged      -> (ready to finalize)
    uncommitted edit to src/       -> (not ready)
    that edit committed            -> (not ready)

The middle row matters: the currency check notices a working-tree change, not merely a committed
one.

## Suite

`cargo test`: 2358 unit + 405 integration, 0 failed. `cargo fmt --check` clean. `specsync check
--strict`: 106/106 files, 0 warnings.

## Not covered

Whether `ship` now succeeds end to end. It does not — the scoped-review gate still refuses, by
design, and that is a separate change. This one makes readiness correct; it does not make the whole
path passable.
