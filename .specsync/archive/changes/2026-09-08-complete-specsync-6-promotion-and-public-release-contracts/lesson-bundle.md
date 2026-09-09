# Lesson bundle — complete-specsync-6-promotion-and-public-release-contracts

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Complete SpecSync 6 promotion and public release contracts
- **Kind**: BugFix
- **Specs**: cli_args, change
- **Paths**: .github/workflows/release.yml, .github/scripts/test-validate-release-candidate.py, .github/scripts/validate-release-candidate.py, src/cli.rs, specs/cli_args/cli_args.spec.md, specs/cli_args/requirements.md, specs/cli_args/context.md, specs/cli_args/tasks.md, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/tasks.md, README.md, SECURITY.md, CHANGELOG.md, site/src/content/docs/cli.md, docs/ci-confidence.md
- **Acceptance**: Promotion and final publication accept exactly the current required Ubuntu and macOS evidence set, and reject missing, duplicate, extra, failed, or mixed-identity evidence without bypassing release protections.
- **Acceptance**: Executable regression controls exercise both workflow evidence-count guards and fail when the obsolete three-record expectation is restored.
- **Acceptance**: CLI help and canonical review contracts distinguish a digest-bound reviewer claim from separately configured authenticated provenance, with no runtime identity or release gate weakened.
- **Acceptance**: README and CLI examples use supported slug workflows and valid supersede flags; security support and release notes describe the 6.0 rollout without claiming an unpublished release already shipped.
- **Acceptance**: The exact final tree passes required local and hosted verification, Trust, provenance, independent review, and a fresh RC qualification before stable publication.

## Evidence

- Verification commit: `011d711e3c262a9c3a6b7226553c7ce471bc8f51`
- Base commit: `ffba9a32b664a7b3308350c162f0ac808f24efe3`
- Verified by: `specsync check --spec change --spec cli_args`

## From the change's context.md

# Context

The independent Claude Fable 5.1 review ran through CorvidLabs Rune on public PR761 snapshot456e1a62 and returned conditional GO at about70 percent subjective confidence. It found promotion-count drift: REQUIRED_PLATFORMS is Ubuntu/macOS, but both promotion and publication shell guards require three records. Both existing guards were executed locally with two receipt filenames and exited1. RC16 has real successful receipts for both required platforms at ffba9a32. No stable release has been published.

This change also corrects remaining authoritative public surfaces: the CLI and canonical contract overclaim local review authentication, README/CLI examples lag the slug workflow, SECURITY omits6.x, and6.0 release notes remain under Unreleased. The separate approved PR761 guidance correction is in progress and must be merged first; rebase this work onto its merged tree before implementation verification. Do not overwrite that work.

## Implementation progress

Both workflow guards now require the existing Ubuntu/macOS evidence set. Fifty-two release-validator tests pass; restoring the old count in either guard fails its corresponding regression. Built CLI help and existing same-actor/pass-block and exact-succession tests pass. Site lint, 23 site tests, and the redirect build pass. Changed Markdown was rendered separately under /private/tmp/specsync-promotion-rendered; the 6.0 release heading is unique and undated, and release-version validation passes. Full local verify is running; canonical materialization, final baseline integration, and release gates are still pending.

The approval CLI initially rejected malformed delta headings; their syntax was corrected without changing the approved wording, and the explicit human scope approval is now successfully recorded as 0xLeif. No self-granted approval or review was recorded.

Separate finding: all three published executable demos fail with retired IDs. Their repair has its own prepared scope under /private/tmp/specsync-6-examples and is awaiting approval. PR761 correction at65de8d9d has all hosted checks green but still awaits the requested human implementation review before final archival and merge.

## From the change's design.md

# Design

Keep the implementation narrow. Correct both shell evidence-count guards to match the existing required platform set. Add executable tests extracting each actual guard and running it with zero/one/two/three receipt names, using the validator's REQUIRED_PLATFORMS length as the acceptance oracle. Existing validator tests retain rejection of duplicate platforms, failed outcomes, mixed identities, missing fields, and extra records. Restore the obsolete count separately in each guard as a negative control and require failure of its regression.

Only help wording changes in src/cli.rs; command parsing, reviewer validation, persisted schema, state transitions, timeout behavior, and provenance policy are unchanged. Update canonical descriptions and the matching requirement to state the actual authentication boundary explicitly. The stored provider declaration remains format-validated metadata, not a claimed identity check.

Release version remains6.0.0. Immutable tags, release token permissions, branch protections, supported-platform set, and required workflows do not change. Actual stable publication follows qualification of a fresh candidate at the final merged tree.

## From the change's testing.md

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

## Where these lessons go

- `specs/cli_args/context.md`
- `specs/change/context.md`
