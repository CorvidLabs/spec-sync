---
change: the-handoff-must-not-claim-a-commit-it-did-not-check
artifact: testing
---

# Testing

`handoff_verifying_follows_evidence_currency` already requires "verification is current" and "independent review". It will also reject any reason that contains "committed".

`handoff_approved_and_implementing_with_a_clean_tree_are_safe` will reject a reason that contains "committed" and still require `` `change check` ``.

Run `cargo test --bin specsync -- handoff_verifying_follows_evidence_currency -- --exact` and `cargo test --bin specsync -- handoff_approved_and_implementing_with_a_clean_tree_are_safe -- --exact`. One filter per invocation.
