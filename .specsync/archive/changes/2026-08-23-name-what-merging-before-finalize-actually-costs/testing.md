---
change: name-what-merging-before-finalize-actually-costs
artifact: testing
---

# Testing

## What is pinned

`the_merge_warning_names_the_cost_to_earlier_accepted_changes` asserts, for both variants:

- "every earlier accepted change sharing a delivery input" — WHOSE work is blocked
- "from archiving" — WHAT is blocked
- "finalized or those are reopened" — BOTH exits

plus that each variant keeps its own opening ("before finalize" / "still active").

It is a discriminator: the pre-change wording contains none of those phrases, so it fails on any
binary carrying the old text. It is also the guard against the likeliest regression — a future
refactor shrinking the message back to "strands the change", which reads fine and silently
under-prices the decision again.

## Confirmed no test asserted the old wording

`grep` for "orphans verification evidence and strands the change" across `src/` and `tests/`
returns nothing outside the sites being changed, so this is not a case of updating an assertion to
match new behaviour.

## Suite

`cargo test`: 2355 unit + 405 integration, 0 failed. `cargo fmt --check` clean. `specsync check
--strict`: 62 specs, 0 warnings, 106/106 files.

## Not covered

Whether the corrected warning changes anyone's behaviour, and whether any existing stranded pile
clears. Neither is testable here, and the second is an open question on #688 that one unrun
command would settle.
