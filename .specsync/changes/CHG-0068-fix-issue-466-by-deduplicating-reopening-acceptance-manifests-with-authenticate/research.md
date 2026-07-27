---
change: CHG-0068-fix-issue-466-by-deduplicating-reopening-acceptance-manifests-with-authenticate
artifact: research
---

# Research

## Current Data Flow

`reopen_change` loads the accepted `verification.json`, authenticates the closing approval, derives
the stale and current acceptance-input digests, and appends a `ReopenRecord` to `approvals.json`.
That record owns a full `VerificationRecord`, whose optional `acceptance_manifest` contains every
signed input entry. `ApprovalLedger` currently derives Serde directly, so unrelated later approval
writes also reserialize the full reopening history.

`acceptance_manifest_digest` already validates manifest shape before producing the signed digest.
Lifecycle paths under `.specsync/changes/` and `.specsync/archive/` are volatile project inputs, so
storing immutable manifest objects inside the change workspace does not recursively add those
objects to later acceptance manifests.

## Alternatives Considered

Storing only the prior verification digest was rejected because historical checks still need the
manifest to validate exact owners and reconstruct stale/current input evidence. Compressing
`approvals.json` was rejected because it leaves repeated logical data, produces opaque diffs, and
does not provide content reuse. Rewriting all old events was rejected because immutable historical
evidence should not be bulk re-signed or reformatted.

Content-addressed objects preserve inspectability, permit deterministic reuse, move only the large
payload, and support strict authentication using existing manifest digests.

## Prerequisite

The 393-entry characterization uses overlapping parent and child delivery scopes. CHG-0067 fixed
the duplicate stage-zero Git index handling that previously blocked this reproduction.
