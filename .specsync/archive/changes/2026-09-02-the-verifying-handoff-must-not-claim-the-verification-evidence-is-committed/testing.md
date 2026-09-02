---
change: the-verifying-handoff-must-not-claim-the-verification-evidence-is-committed
artifact: testing
---

# Testing

`handoff_verifying_follows_evidence_currency` already walks Verifying through stale evidence
(conditional), current evidence awaiting review (safe), and current review (safe). It gains
one assertion per safe arm: the reason must not contain "verification is committed". That
fails on the current binary — the awaiting-review arm carries exactly that phrase — and passes
after the reword.

Discriminator: `handoff_verifying_follows_evidence_currency` (fails before, passes after).
Control: `handoff_follows_the_lifecycle_and_ignores_uncommitted_lifecycle_evidence`, which
drives a real repository through the same state with `verification.json` uncommitted and
still expects `safe` — the verdict is untouched.

Run: `cargo test --release --bin specsync handoff` (12 tests) and the integration test
`status_prints_a_handoff_line_and_json_carries_it`.
