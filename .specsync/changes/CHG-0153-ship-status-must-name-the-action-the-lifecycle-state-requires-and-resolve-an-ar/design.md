---
change: CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar
artifact: design
---

# Design

## The single decision: reuse the resolver, do not add one

`commands/change.rs` needed to find a change's artifacts. Three constructions were available:

| option | cost |
|---|---|
| keep `root.join(".specsync/changes").join(id)` | the defect — wrong the moment a change archives |
| add an `evidence_dir` helper in `commands/` | a **third** idiom beside `change_dir` and `find_change_dir`; the exact shape that has recurred eight times this release |
| expose `find_change_dir` | removes two parallel implementations, adds none |

The third is the only one that makes the codebase smaller in the sense that matters. It also
inherits `find_change_dir`'s existing behaviour for free: id validation, and refusal when a
change appears in both active and archive locations.

Cost of exposing it: one row in the `change` module's Public API section, because the
effective-contract check requires every exported item to be documented. That is a real cost and
the right one to pay.

## Why lenient reads are a design choice, not laziness

`ship-status` and `ship` are inspection commands. Their value is highest exactly when a
repository is in a bad state, which is when their inputs are most likely to be damaged. A strict
parse inverts that: the worse the repository, the less the tool will tell you.

Measured on a shim with strict `?`: a truncated or conflict-marked archived `verification.json`
takes both commands from rc=0 to rc=1. That is a fix for one problem creating a worse one.

So: unreadable or unparseable evidence reads as *no evidence recorded*. That is honest — the
command genuinely cannot see verification — and it keeps every other field renderable.

Note this is **not** the release's defect theme in reverse. The theme is an empty category read
as an absence of problems. Here the absence is reported as an absence, on a line whose whole
purpose is to say what evidence exists; nothing is inferred to be fine.

## Why the shipping-window test is on state, not on stage

`ship_next` previously derived from `current_stage["action"]`, i.e. from where the *tip* is. The
lifecycle state is what determines whether the ship lane has anything to say at all. Draft and
Accepted are before it; Archived is after it. Testing the state keeps the rule stateable in one
sentence — *the lane may narrow the next action, never contradict the lifecycle* — which is the
same rule #626 established for CI lane classification.

## Rejected: keeping the blocker in the line

First attempt rendered `{action} — blocked: {blocker}`. Drill 053 rejected it, correctly. The
blocker already prints on its own `Blocker:` line, so the suffix was pure duplication, and it
left a blocker restatement inside a line whose contract is "a command you can run". Deleted
rather than reworded.
