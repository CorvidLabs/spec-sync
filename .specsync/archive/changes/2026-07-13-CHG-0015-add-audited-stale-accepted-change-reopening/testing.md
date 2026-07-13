---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: testing
---

# Testing

`REQ-change-017` is covered by `change::tests::stale_accepted_change_reopens_with_audited_evidence_and_reaccepts`, `change::tests::reopen_rejects_current_evidence_and_requires_explicit_audit_fields`, `cli::tests::change_reopen_requires_and_collects_audit_inputs`, and `change::stale_accepted_change_reopens_through_cli_with_deterministic_audit_json`.

The regression sequence proves initial acceptance, scoped input mutation, strict stale failure, audited reopen, continued strict failure, fresh verification, fresh closing approval, strict pass, preserved prior evidence, required audit inputs, deterministic JSON, and rejection of current evidence. Release validation uses the full Fledge repository lane and strict SpecSync scoring/checks.
