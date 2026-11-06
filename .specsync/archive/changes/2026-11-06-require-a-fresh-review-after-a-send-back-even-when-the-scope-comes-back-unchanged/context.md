---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: context
---

# Context

Issue #960, from the Codex review of #944 (#914). A scoped review binds a definition approval by digest. When a reviewed change is sent back and re-approved with an unchanged scope, the new approval has the same digest. If `change check` then runs without `--commit`, `HEAD`, the execution digest and the project-input digest stay the same too. The earlier review then binds the post-send-back approval by digest, `scoped_review_currency` returns `Current`, and `change ship`/`finalize` pass without a fresh review.

Reproduced on main 8a251d7a through the built binary under workflow v2 (`a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`): after `approve → check --commit → review → send-back → approve → check`, status reports `scoped_review_current: true` and the next action `finalize`.

Leif's intent, captured verbatim as CHANGE-12.e: "After I send a change back, it needs a fresh review before it can ship, even when its scope comes back unchanged." This was main's behaviour before #944 too; #914's approved scope named it out of scope, and Leif chose to fix it in its own change.

Out of scope: review-history validity (#914), the send-back command's other effects, and what a review must match beyond being recorded after the latest send-back.
