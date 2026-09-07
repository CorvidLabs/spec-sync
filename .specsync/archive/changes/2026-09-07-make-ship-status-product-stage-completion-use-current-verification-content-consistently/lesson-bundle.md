# Lesson bundle — make-ship-status-product-stage-completion-use-current-verification-content-consistently

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make ship-status product-stage completion use current verification content consistently
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, specs/cmd_change
- **Acceptance**: Product-stage completion uses the same verification-content currency predicate as readiness; unchanged verified content after rewritten ancestry reports product verification done; altered content still reports incomplete; text and JSON agree; review currency and finalization policy remain enforced.

## Evidence

- Verification commit: `ab424040b5252377cd1103329983eb65077cf6df`
- Base commit: `249a3e0db7e083569440e864501c416072feb5a6`
- Verified by: `specsync check --spec cmd_change`

## From the change's context.md

# Context

Issue #745 identifies a confirmed disagreement in src/commands/change.rs. ship_status_report computes verification_current with recorded_verification_is_current, but ship_stages still receives verification_ancestor and derives product_done from history. The action text also claims ancestry. The fix will pass the existing currency result to the stage projection and sweep sibling presentation paths for the same contradiction.

This is a proposed scope, not approved implementation. Base: merged #757 at 249a3e0d. Candidate rc.15 remains unchanged on a separate branch. No persisted evidence format or domain review policy is changed. #694 remains a separate design decision.

## From the change's testing.md

# Testing

Add regressions around the real ship-status report, not only a boolean helper: current content with rewritten ancestry; stale content despite ancestor evidence; missing evidence; and a stale/unavailable review that remains ineligible for finalize. Assert the product stage and readiness do not contradict each other on the verification term while allowing review to independently prevent finalization. Assert explanatory text does not claim ancestor evidence when ancestry is absent.

Run focused cargo test --bin specsync commands::change::tests, strict cmd_change contract checks, the mandatory pre-push gate and full verification/Trust before completion. A stale-content control must fail against an unconditional done implementation.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-cmd-change-016 | `ship_status_and_finalize_agree_after_a_squash_that_preserves_content`, `ship_status_product_stage_rejects_stale_content_with_ancestor_evidence`, `ship_status_product_stage_rejects_missing_verification`; all 16 command-module tests passed after both discriminator cases failed on the previous implementation. |

## Where these lessons go

- `specs/cmd_change/context.md`
