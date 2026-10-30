# Lesson bundle — keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep check fix export rows in one contiguous table for issue 768
- **Kind**: BugFix
- **Specs**: cmd_check
- **Paths**: src/commands/check.rs, tests/integration/fix.rs, specs/cmd_check/cmd_check.3md, specs/cmd_check/cmd_check.spec.md, specs/cmd_check/requirements.md, specs/cmd_check/tasks.md, specs/cmd_check/testing.md
- **Acceptance**: For issue 768 case 1, new export rows extend one contiguous table using its own width; prefer the first Name header and otherwise the first table; preserve surrounding content; pass strict recheck and repeated-fix idempotence. Case 2 remains out of scope.

## Evidence

- Verification commit: `06a4600a191e579a6d028593aadf5a528be570e4`
- Base commit: `9003fad7ec4dea3938179d076248b48f29b6ed48`
- Verified by: `specsync check --spec cmd_check`

## From the change's context.md

# Context

## Evidence and scope
Issue #768 remains open and unassigned. Its latest maintainer comment says #770 fixed only case 3; cases 1 and 2 remain. At selection, the only open PR was excluded #800. Fresh main was 9003fad7ec4dea3938179d076248b48f29b6ed48. No generate binding files are in scope.

Read cmd_check.spec.md and all companions, including authoritative cmd_check.3md. Preserve deterministic local fixes, code examples, and export-kind routing.

## Narrowed scope after reproduction
Case 1 is this patch. Case 2 remains open: the insertion-only fix still duplicates an existing symbol because parser fence blanking hides its documentation. No parser files are changed here. The initial two-test red log records discovery; the final regression isolates case 1 with nine layout/header combinations.

## Ownership recheck
Main remained 9003fad on 2026-10-11. Open PRs are now #800 and #814; #814's file list has no overlap with this patch. Issue #768 comments and assignment are unchanged.

## Human scope authorization
The user was asked: "For #768, the scope is just the wrong-table insertion bug, its regression tests, and matching spec/docs. The fenced-code parsing bug stays separate. Approve that scope and a draft PR once validation is sorted?"
The user (Sentinel_fa7dbbab35548191ad234f735456fa69) answered: "yes alweasy test locally first and make sure you are 95% confident on your fix or change and that it doesn't cause any regressions".
This authorizes the bounded case-1 scope and draft publication after strong local validation. It is not implementation review, acceptance, merge permission, or a literal measured probability. No case-2 parser changes are included.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-cmd-check-015 | `tests/integration/fix.rs::fix_768_uses_one_contiguous_export_table` covers nine table/header layouts and asserts width, immediate placement, exact preservation, strict recheck and idempotence; all 25 fix tests pass. |

## Regression evidence
`cargo test --locked --offline --test integration fix_768 -- --nocapture` failed both initial tests against unchanged main implementation. Log: /tmp/specsync-768-red.log.

## Planned gates
Focused integration fix suite; fledge lanes run verify; fledge lanes run pre-push; fledge trust verify; Attest only after successful verification. Record actual results before handoff.

## Initial verification results (historical)
- Initial regression failed on unchanged main implementation; /tmp/specsync-768-red.log.
- Final focused fix suite: 25 passed, including nine multi-table layout/header cases; /tmp/specsync-768-focused.log.
- Pre-push gate passed: formatting, types, source-built strict validation; 62 specs, zero warnings, 121/121 source files covered; /tmp/specsync-768-pre-push.log.
- Module quality score: 100/100.
- Release-candidate validator tests: 52 passed. CI-gate tests: 29 passed.
- Full verify lane failed: 2540 unit tests passed, three lifecycle tests failed with Git repository detection errors. All three passed individually with the exact main implementation. This does not establish the parallel full suite is healthy. Logs: /tmp/specsync-768-verify.log and /tmp/specsync-768-baseline-*.log.
- Full retry: 2487 unit tests passed, 56 failed; /tmp/specsync-768-verify-retry.log. Full integration suite: 437 passed, two lifecycle Git-discovery failures (`reopened_owner_correction_is_deterministic_through_json_cli` and `stale_accepted_change_reopens_through_cli_with_deterministic_audit_json`); /tmp/specsync-768-integration.log. No full-suite success or tests-passed attestation is claimed.
- Trust gate passed with progressive/degraded provenance; Attest policy is unsatisfied because full verification did not pass. No tests-passed attestation was recorded.
- Persistent logs: /home/parallels/repos/spec-sync-768-evidence/.

## Initial lifecycle status before human scope approval
The scope definition is complete but unapproved. Scope approval, scoped check/commit evidence, human review, finalization, publication and merge remain pending. `change audit` does not count this draft as approved delivery coverage. No human approval or acceptance was fabricated.

## Matched diagnosis and resolved local failures
The initial failed runs remain historical evidence. A fresh clean main checkout at 9003fad and preserved fix commit a305444a were rebuilt in separate target directories and tested sequentially with identical environment (CARGO_NET_OFFLINE=true, RUST_TEST_THREADS=4, TMPDIR and Git overrides unset). Both used:
- `cargo test --locked --bin specsync -- --test-threads=4`: 2543 passed on each, zero failures.
- `cargo test --locked --test integration -- --test-threads=4`: main 438 passed; fix 439 passed; zero failures. The additional test is the nine-case multi-table regression.
- Serial confirmation of 13 earlier failing unit tests and two integration tests passed on both commits using `--test-threads=1 --exact`.

The first shared-target comparison was invalidated because Cargo reused the baseline integration artifact (438 instead of 439); only the separate-target results above establish the comparison.

Read-only monitoring observed transient empty `/tmp/.git` directories. After testing ended, two controlled harmless sandboxed commands each created then removed that marker for roughly 60ms. A fixture without an ancestor marker succeeded; adding an empty ancestor `.git` reproduced the Git-detection error. Lifecycle source is byte-identical between the two commits. This explains why parallel reads through the sandbox could interfere with temporary-project tests; it is not a case-1 source regression. No shared Git configuration or security setting was modified. Future qualification uses a private TMPDIR outside /tmp.

Detailed logs and exact failing-test sets: `/home/parallels/repos/spec-sync-768-evidence/comparison.json`, `verified-matched/`, `serial/`, `ancestor-marker-probe.json`, and `sandbox-marker-probe.jsonl`.

## Final local qualification after diagnosis
`TMPDIR=/home/parallels/repos/spec-sync-768-evidence/scratch CARGO_NET_OFFLINE=true fledge lanes run verify` passed all six lane steps: formatting, lint and types; 2543 unit tests; 439 integration tests; release build; strict spec validation; 52 release-candidate validator tests; and 29 CI-gate validator tests. Strict validation covers 62 specs with zero warnings and 121/121 source files. Log: `/home/parallels/repos/spec-sync-768-evidence/final-verify-scratch.log`.

A preliminary private directory named `test-tmp` exposed a generator test that checks the full path for the word `test`. The exact test failed under that path on both clean main and this fix, and passed on both with the path renamed to `scratch`. Only the per-command environment changed; generator source and global settings were not changed. The complete corrected lane then passed.

The user has now approved the bounded case-1 scope and conditional draft publication. Supported lifecycle approval and scoped verification are recorded. Human implementation review, acceptance/finalization, and merge remain pending. Confidence is supported by the focused red/green regression, matched baseline controls, and full local qualification; no numerical probability guarantee is asserted.

## Where these lessons go

- `specs/cmd_check/cmd_check.3md`
