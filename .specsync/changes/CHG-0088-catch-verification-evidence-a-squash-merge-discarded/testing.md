---
change: CHG-0088-catch-verification-evidence-a-squash-merge-discarded
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-020` | The preflight script was extracted with PyYAML and run under GitHub Actions' literal default invocation, `bash --noprofile --norc -eo pipefail`, across six repository states. It exits 0 for null JSON, malformed JSON, a null commit and an ancestor commit; it exits 1 for a commit absent from the repository and for a present non-ancestor commit. The absent case is the one this change adds. |

## Manual verification

Observed on main at 121168ac, where CHG-0087 had been merged before it was
finalized:

| job | result | time |
|---|---|---|
| Lifecycle preflight | success | 7s |
| Lifecycle gate | failure | 4m28s |

The preflight passed by skipping the orphaned commit. The gate caught the same
condition four and a half minutes later. After this change the preflight fails
in seconds on that state.
