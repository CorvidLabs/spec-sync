---
change: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
artifact: testing
---

# Testing

## Regression evidence
`cargo test --locked --offline --test integration fix_768 -- --nocapture` failed both initial tests against unchanged main implementation. Log: /tmp/specsync-768-red.log.

## Planned gates
Focused integration fix suite; fledge lanes run verify; fledge lanes run pre-push; fledge trust verify; Attest only after successful verification. Record actual results before handoff.

## Current verification results
- Initial regression failed on unchanged main implementation; /tmp/specsync-768-red.log.
- Final focused fix suite: 25 passed, including nine multi-table layout/header cases; /tmp/specsync-768-focused.log.
- Pre-push gate passed: formatting, types, source-built strict validation; 62 specs, zero warnings, 121/121 source files covered; /tmp/specsync-768-pre-push.log.
- Module quality score: 100/100.
- Release-candidate validator tests: 52 passed. CI-gate tests: 29 passed.
- Full verify lane failed: 2540 unit tests passed, three lifecycle tests failed with Git repository detection errors. All three passed individually with the exact main implementation. This does not establish the parallel full suite is healthy. Logs: /tmp/specsync-768-verify.log and /tmp/specsync-768-baseline-*.log.
- Full retry: 2487 unit tests passed, 56 failed; /tmp/specsync-768-verify-retry.log. Full integration suite: 437 passed, two lifecycle Git-discovery failures (`reopened_owner_correction_is_deterministic_through_json_cli` and `stale_accepted_change_reopens_through_cli_with_deterministic_audit_json`); /tmp/specsync-768-integration.log. No full-suite success or tests-passed attestation is claimed.
- Trust gate passed with progressive/degraded provenance; Attest policy is unsatisfied because full verification did not pass. No tests-passed attestation was recorded.
- Persistent logs: /home/parallels/repos/spec-sync-768-evidence/.

## Lifecycle limit
The scope definition is complete but unapproved. Scope approval, scoped check/commit evidence, human review, finalization, publication and merge remain pending. `change audit` does not count this draft as approved delivery coverage. No human approval or acceptance was fabricated.
