---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: context
---

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
