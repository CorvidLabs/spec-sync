# Lesson bundle — ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change.rs, src/commands/change.rs
- **Acceptance**: ship-status readiness consults scoped-review currency, reports it as current/stale/unavailable, blocks on a decided stale review naming what moved, never reports an unavailable guarantee as satisfied, and agrees with finalize on the same tree; a genuinely current review still reaches ready_to_finalize: true

## Evidence

- Verification commit: `e450d15a8e862d04f90a6c08d690866b4bca8e5a`
- Base commit: `7df407728de3ac6458ef8807e79bbadb51da3324`
- Verified by: `cargo test change::`, `cargo test commands::change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

## What was observed

One command apart, with only a read-only `ship-status` between them (#743, found by sandbox
drill 008 against v6.0.0-rc.10 and confirmed on `main` at `db1f4ac9`):

```
finalize (rc=1): error: independent scoped review is stale; open or update the PR so
                 `SpecSync scoped review` can run
ship-status:     ready_to_finalize: true
                 Next: run `specsync change ship <id>`
```

The tool recommended a verb and then refused it.

## Why it happened

`ready_to_finalize` was a conjunction of `verification_present && verification_current &&
review_present && blockers.is_empty()`, and `review_present` was `review_path.is_file()` —
existence. `finalize` additionally requires the review to be CURRENT, through
`scoped_review_is_current`. That predicate was not missing: `ChangeSummary.scoped_review_current`
already calls it. `ship-status` was simply the one caller that never did.

The asymmetry is visible in the comment that sat directly above the defect. #689 rebuilt readiness
as a CONTENT question and fixed the **verification** half; the **review** half was left asking only
whether a file was on disk. Readiness never asked, so it never received a negative answer, and the
silence read as a pass — the shape this release has been bitten by repeatedly.

## What was already ruled out

**Adding `&& scoped_review_current` to the conjunction.** After a squash,
`scoped_review_is_current` walks descendants of `review.implementation_commit`, a commit the squash
destroyed. Measured on this repository: 0 of 107 archived reviews would pass that walk, because
archiving relocates the workspace out from under the walk's own allowlist. The naive fix makes every
squash-merging repository permanently unable to reach `ready_to_finalize` — precisely what #689
removed from the verification half. Trading a false green for a permanent false red is not an
improvement.

## The distinction the code could not make

`scoped_review_is_current` returns `bool`. It collapsed two different answers:

| review currency | before | after |
|---|---|---|
| current | ready | ready |
| **stale** — content genuinely changed | reported ready, finalize refused | not ready, naming what changed |
| **unavailable** — squash destroyed the anchor | reported ready, finalize refused | reported `unavailable`, never satisfied |

Its git sub-check `review_commit_is_current_checked` already returned `Result<(), String>` with
distinct reasons, but both of its callers discarded the reason with `.is_ok()`. So the ingredients
for the distinction were on disk and the function threw them away.

## What is deliberately NOT decided here

Whether an unavailable descendant guarantee should block finalization at all. That is #694, which
has three live options and is being decided deliberately rather than by whichever patch lands. This
change stops readiness reporting `unavailable` as `true` — #694's own stated standard, "an
unavailable guarantee reported as a satisfied one is worse than the current failure", applied to the
one caller that violated it — without settling what should then happen.

## From the change's design.md

# Design

## The seam

A three-valued `ScopedReviewCurrency` in `src/change.rs`:

- `Current` — every recorded binding still agrees with the tree.
- `Stale(reason)` — a decided negative: a bound digest moved, or the descendant walk ran and
  inspected a commit that changed a path outside this change's own lifecycle records.
- `Unavailable(reason)` — the question could not be answered at all.

`scoped_review_is_current` becomes `scoped_review_currency(..) == Current`, so `finalize` and
`accept` keep the exact predicate they had; there is one implementation, not two.
`review_commit_is_current_checked` becomes `review_commit_currency`, whose walk body is factored
into `review_commit_walk` returning `Ok(None)` (clean) / `Ok(Some(reason))` (violation) /
`Err(reason)` (could not run) — the three-way split the old `Result<(), String>` could not express.

`recorded_scoped_review_currency` is the public entry, the scoped-review twin of the existing
`recorded_verification_is_current`.

## Why `Unavailable` is a warning and `Stale` is a blocker

`Stale` is a decided negative with a reachable repair, so it renders exactly like the verification
half #689 fixed: a blocker naming what moved and the re-review that fixes it.

`Unavailable` renders as a warning plus `ready_to_finalize: false`. A blocker would assert that an
unobtainable guarantee ought to block, which is #694's open question; a warning says "readiness
cannot confirm this" and names the re-review that re-anchors it. Either way the two commands agree,
which is the property that must hold under all three of #694's options.

## Why the reader-facing labels live in the command layer

`ShipReviewStatus` in `src/commands/change.rs` extends the domain's three answers with `Missing`
and `Unreadable`, which are questions about the artifact rather than about its currency. Keeping
those two variants out of `ScopedReviewCurrency` is what stops the domain type from becoming a
presentation type.

## Rejected

**`&& scoped_review_current` on the conjunction.** Correct for the stale case and catastrophic for
the unavailable one: it turns every squash-merging repository permanently unfinalizable, which is
the #689 regression with the sign flipped. The control test exists to keep anyone from reaching for
it again.

## From the change's testing.md

# Testing

All three new assertions were run against a binary built from a SEPARATE checkout of unfixed `main`
(a fresh clone at `7df4077`), never by reverting in place.

## DISCRIMINATOR — `ship_status_and_finalize_agree_when_the_review_is_stale_by_content`

- `REQ-cmd-change-007`: readiness and finalization reach the same conclusion.
- Records a review, changes the implementation, then re-runs `check` so that the VERIFICATION half
  is healthy again — without that isolation the verification blocker alone would make readiness
  false and the test would prove nothing, because every content digest the review checks is also
  checked by verification.
- Asserts AGREEMENT (`ready_to_finalize == finalize.is_ok()`), not `ready_to_finalize == false`.
  All three of #694's options end with the two agreeing, so the narrower assertion would go red the
  day #694 is resolved the other way.
- Verbatim failure on the control binary:

```
assertion `left == right` failed: ship-status and finalize disagree about the same change in the
same second: ready_to_finalize=true,
finalize=Err("independent scoped review is stale; open or update the PR so `SpecSync scoped review`
can run"), report={... "ready_to_finalize":true, "review_present":true, "blockers":[] ...}
  left: true
```

## DISCRIMINATOR — `ship_status_and_finalize_agree_after_a_squash_that_preserves_content`

- `REQ-cmd-change-007`, `REQ-change-046`: an unavailable guarantee is never reported as satisfied.
- The #689 test, relabelled. Its verification half is asserted unchanged (no verification blocker
  while `verification_ancestor_of_head` is false). Its `ready_to_finalize: true` assertion was the
  defect written down as an expectation, and is now an agreement assertion.
- Verbatim failure on the control binary: identical shape, `ready_to_finalize=true` beside
  `finalize=Err("independent scoped review is stale; ...")`.

## CONTROL — `ship_status_is_ready_when_the_scoped_review_is_current`

- Honest label: this PASSES on the unfixed binary, and that is the point. Without it, "make
  readiness stricter" clears both discriminators while breaking every healthy change.
- Confirmed passing against the control binary in the same run that failed the two above
  (`1 passed; 2 failed`).

## End to end

Sandbox drill 008's declared PENDING GATE (#694) now clears against the release binary:

```
OK: squash-merge before finalize strands the change and fails closed
OK: ship-status still names the recovery (product_tip: change check --commit)
OK: ship-status and finalize agree (finalize rc=1, ready_to_finalize=0)
OK: re-checking on the squashed tip recovers the change and finalize succeeds
DRILL 008 PASSED
```

The last line matters as much as the gate: the recovery is still reachable, so this is not a
permanent red.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
