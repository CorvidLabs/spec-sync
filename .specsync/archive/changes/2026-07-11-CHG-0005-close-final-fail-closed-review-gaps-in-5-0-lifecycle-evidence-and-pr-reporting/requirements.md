---
change: CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting
artifact: requirements
---

# Requirements

## REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval, history, and verification evidence before using it.

Acceptance Criteria
- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed, fresh verification evidence in CI and local checks.

## REQ-cmd-comment-001

Generated pull-request comments SHALL include SDD lifecycle failures in their status and remediation details.

Acceptance Criteria
- SDD errors and warnings appear in the rendered comment alongside canonical spec validation.
- An SDD-only failure produces a failing comment status.
