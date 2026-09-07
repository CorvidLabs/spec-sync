---
change: make-ship-status-product-stage-completion-use-current-verification-content-consistently
artifact: context
---

# Context

Issue #745 identifies a confirmed disagreement in src/commands/change.rs. ship_status_report computes verification_current with recorded_verification_is_current, but ship_stages still receives verification_ancestor and derives product_done from history. The action text also claims ancestry. The fix will pass the existing currency result to the stage projection and sweep sibling presentation paths for the same contradiction.

This is a proposed scope, not approved implementation. Base: merged #757 at 249a3e0d. Candidate rc.15 remains unchanged on a separate branch. No persisted evidence format or domain review policy is changed. #694 remains a separate design decision.
