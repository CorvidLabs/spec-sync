---
change: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-check-015 | `tests/integration/fix.rs::fix_768_uses_one_contiguous_export_table` covers nine table/header layouts and asserts width, immediate placement, exact preservation, strict recheck and idempotence; all 25 fix tests pass. |

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

## Matched diagnosis and resolved local failures
The initial failed runs remain historical evidence. A fresh clean main checkout at 9003fad and preserved fix commit a305444a were rebuilt in separate target directories and tested sequentially with identical environment (CARGO_NET_OFFLINE=true, RUST_TEST_THREADS=4, TMPDIR and Git overrides unset). Both used:
- `cargo test --locked --bin specsync -- --test-threads=4`: 2543 passed on each, zero failures.
- `cargo test --locked --test integration -- --test-threads=4`: main 438 passed; fix 439 passed; zero failures. The additional test is the nine-case multi-table regression.
- Serial confirmation of 13 earlier failing unit tests and two integration tests passed on both commits using `--test-threads=1 --exact`.

The first shared-target comparison was invalidated because Cargo reused the baseline integration artifact (438 instead of 439); only the separate-target results above establish the comparison.

Read-only monitoring observed transient empty `/tmp/.git` directories. After testing ended, two controlled harmless sandboxed commands each created then removed that marker for roughly 60ms. A fixture without an ancestor marker succeeded; adding an empty ancestor `.git` reproduced the Git-detection error. Lifecycle source is byte-identical between the two commits. This explains why parallel reads through the sandbox could interfere with temporary-project tests; it is not a case-1 source regression. No shared Git configuration or security setting was modified. Future qualification uses a private TMPDIR outside /tmp.

Detailed logs and exact failing-test sets: `/home/parallels/repos/spec-sync-768-evidence/comparison.json`, `verified-matched/`, `serial/`, `ancestor-marker-probe.json`, and `sandbox-marker-probe.jsonl`.
