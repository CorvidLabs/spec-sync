# Verification

All simulated human decisions in test fixtures are explicitly synthetic.
Native structural checks do not run the repository product suite.

| Requirement | Evidence |
| --- | --- |
| REQ-change-recovery-retirement-001 | src/change_tests/recovery_retirement.rs: production replacement/custody/retirement/child finalization; dependency refusal; committed/current evidence tamper; missing/stale/unreviewed/circular/unknown input; owner mismatch; injected/interrupted move recovery |
| REQ-change-frozen-recovery-001 | recovery_frozen_added_conflict_preserves_newer_canonical_bytes_and_names_hold; existing stale MODIFIED and materialization controls remain in the full suite |
| REQ-cmd-change-recovery-retirement-001 | Actual published and source-built reader status/audit receipts, explicit nonacceptance and inspected audit scope |
| REQ-cli-args-recovery-retirement-001 | cli::recovery_retire_grammar::recovery_retire_grammar_requires_explicit_authority_and_replacement |

Historical red/green receipts remain in ../product-evidence/.
The latest focused safety run passed 10 tests. The old ee5e reader consumer results
are not final-head qualification. Full verify/strict/pre-push/Trust remain required.
Root owns actual BABY and parent integration qualification. Parent path coverage is complete,
but two unchanged old-exact/current-studio owner differences remain held; no exception added.
