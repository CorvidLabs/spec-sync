---
change: complete-specsync-6-promotion-and-public-release-contracts
artifact: testing
---

# Testing

## Targeted verification

| Requirement | Test | Evidence |
|---|---|---|
| REQ-cli-args-015 | Inspect built change review help and existing CLI parsing tests | Passed: built CLI help inspection, existing CLI review/supersede tests, and complete local verify |
| REQ-change-046 | Existing scoped-review claim, same-approver, append-only, blocking, and freshness tests plus source inspection | Passed: existing scoped-review same-actor/pass-block test and complete local verify |

Promotion regression: execute both extracted workflow guard bodies with receipt counts0,1,2,3; accept only len(REQUIRED_PLATFORMS). Restore each old3 guard independently and demonstrate the relevant test fails. Keep existing validator missing/extra/duplicate/failed/mixed-identity negative tests.

Run the complete release-validator test suite and full local verify. Run site lint/test/build and inspect changed Markdown separately because the standalone site build emits redirects. Verify strict specs, coverage, score, exact scoped evidence, Trust, and signed policy. Remote CI and fresh immutable RC qualification remain separate evidence. Never claim the reviewer executed supplied tests.

## Recorded results

Full local `fledge lanes run verify` passed: 2464 unit tests, 415 integration tests, clippy, formatting, type checks, optimized build, strict specs (62 passing, zero warnings, 100% coverage), and all 52 release-validator tests. Log: /private/tmp/specsync-promotion-verify.log. Both count-guard tests passed against the fix; restoring either obsolete guard independently makes its own regression fail (/private/tmp/specsync-promotion-negative-0.log and -negative-1.log). The live publication channels still serve5.2.0 (crates/Homebrew) andRC14 (GitHub prerelease); no6.0 stable is claimed.

Site lint, 23 tests, and redirect build pass; changed Markdown is separately rendered in /private/tmp/specsync-promotion-rendered. Release-version and workflow-runtime-pin validators pass. Final integration with PR761, hosted checks, human review, archival, fresh RC qualification, and stable publication remain separate pending gates.
