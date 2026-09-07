---
change: make-ship-status-product-stage-completion-use-current-verification-content-consistently
artifact: testing
---

# Testing

Add regressions around the real ship-status report, not only a boolean helper: current content with rewritten ancestry; stale content despite ancestor evidence; missing evidence; and a stale/unavailable review that remains ineligible for finalize. Assert the product stage and readiness do not contradict each other on the verification term while allowing review to independently prevent finalization. Assert explanatory text does not claim ancestor evidence when ancestry is absent.

Run focused cargo test --bin specsync commands::change::tests, strict cmd_change contract checks, the mandatory pre-push gate and full verification/Trust before completion. A stale-content control must fail against an unconditional done implementation.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-cmd-change-016 | `ship_status_and_finalize_agree_after_a_squash_that_preserves_content`, `ship_status_product_stage_rejects_stale_content_with_ancestor_evidence`, `ship_status_product_stage_rejects_missing_verification`; all 16 command-module tests passed after both discriminator cases failed on the previous implementation. |
