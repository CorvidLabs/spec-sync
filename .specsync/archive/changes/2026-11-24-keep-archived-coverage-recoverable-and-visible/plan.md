---
change: keep-archived-coverage-recoverable-and-visible
artifact: plan
---

# Plan

## Approach

Use the existing project-locked reopen publication and its durable rollback journal for a narrowly scoped receipt removal. Use one shared typed reader for coverage displayed by the CLI and archive audit.

## Out of scope

Authenticated archive formats, explicit successor rules, covering-owner matching, and automatic cleanup of unexpected or invalid files remain unchanged.

## Steps

1. Reproduce the two failures and verify the valid and absent controls.
2. Add the receipt validation and record its eligible deletion with the reopen transaction.
3. Wire the separate coverage diagnostic to text and JSON, then rerun the original CLI sequence and rollback checks.
4. Materialize the two spec deltas, run scoped and full verification, and prepare the work for implementation review.

## Risks

A separate unlink could strand old evidence or silently consume an unrecognized file. Journal the confined deletion alongside state writes and retain the active archive conflict guard. Archive and receipt validation remain independent.

## Constraints consulted

REQ-change-162 covers recoverable journaled writes. REQ-change-165 covers path confinement. REQ-change-167 keeps temporary coverage separate from terminal archive authentication. The CLI remains a renderer for domain diagnostics.
