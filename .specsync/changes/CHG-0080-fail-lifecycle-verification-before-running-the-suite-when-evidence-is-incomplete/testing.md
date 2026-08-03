---
change: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-049` | `added_block_reapplies_when_content_is_identical` covers convergence and the different-content error; `change_ordinals_identify_independently_allocated_workspaces` covers ordinal identity; `failed_native_verification_is_retryable_with_append_only_history` covers the named command failure and append-only attempt history. |

## Focused regressions

- Re-deriving an already-applied `## ADDED` block returns the source unchanged.
- The same block with different content errors and names `## MODIFIED`.
- `CHG-0078-…` and a second `CHG-0078-…` share an ordinal; `CHG-0079-…` does not.
- Non-ordinal identifiers never raise a collision.
- A failed verification command names itself and its exit code, and still records an attempt.

## Full suite

`cargo test` — 2,181 unit and 333 integration tests passed.
