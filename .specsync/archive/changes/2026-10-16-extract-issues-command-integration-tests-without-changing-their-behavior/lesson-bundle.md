# Lesson bundle — extract-issues-command-integration-tests-without-changing-their-behavior

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Extract issues command integration tests without changing their behavior
- **Kind**: Refactor
- **Paths**: tests/integration/commands.rs, tests/integration/commands/issues.rs, tests/integration/commands/specsync_process.rs, specs/cmd_issues/testing.md
- **Acceptance**: Agent-proposed outcomes: conserve all26 test bodies/attributes/fixtures and shared helper behavior, select identical24macOS tests before/after, pass full native2582unit/462integration, retain unexecuted platform limits, update documented filters and preserve product behavior.

## Evidence

- Verification commit: `e47b6bd5c57e6e3d1a21c286f71b136bf8d49430`
- Base commit: `06d343e630deb7ad14eca93b242f7060b0928646`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

Leif asked to split oversized SpecSync files into focused, tested work and publish reviewable PRs. This candidate extracts one existing integration-test family only. The large process file shrinks from 5,962 to 4,874 lines; all 26 complete test functions, attributes and inline fixture bytes are conserved. Shared helpers, environment isolation and product behavior are unchanged.

Hosted PR #820 initially failed its strict lifecycle audit because the four meaningful changed paths lacked an active change record. The repository guide calls SDD optional, but the actual CI policy requires coverage here. The original agent-proposed draft recorded that scope without weakening CI or inventing human approval. On 2026-10-16 Leif directly approved the prepared scope, conditional on actual local testing and at least 95% engineering confidence. The supported writer recorded actor 0xLeif and the direct response is preserved in ci-cleanup-human-approval.json. Current qualification, actual independent implementation review and supported finalization remain separate evidence-bound gates.

## From the change's testing.md

# Testing

## Actual local evidence

- Baseline receipt `20261016T052706Z-S3-8947fb`: 24 macOS issues tests passed.
- Moved-family receipt `20261016T053423Z-S3-19c014`: the same 24 names passed under `commands::issues::`; 438 other tests filtered out.
- Release compile `20261016T053449Z-S3-dd808a`: release integration executable built; no release-profile test execution claimed.
- Full native `20261016T053719Z-S3-04e12e`: 2,582 unit tests and 462 integration tests passed, plus formatting, lint/types, release build, 64 release-candidate controls and 29 CI-gate controls.
- Released SpecSync 6.1.1 `20261016T055336Z-S3-561998`: forced strict check 62/62, zero warnings, path coverage 121/121 and LOC 148111/148111.
- Independent GPT review proved all 26 complete functions/attributes/fixture slices and remaining helpers/isolation conserved; exact four source hashes are preserved in the coordinator's report.
- Pre-push `20261016T060033Z-S3-9900cf` passed. Actual committed range `20261016T060125Z-S3-2dbdad` passed Trust at risk 23/proceed, with unchanged-policy unsigned agent provenance subsequently verified and published. Post-publication normal Trust `20261016T060319Z-S3-9a3bab` passed.

## Hosted result and limits

At original head `06d343e630deb7ad14eca93b242f7060b0928646`, hosted Lifecycle gate job `116836835319` failed because these four meaningful paths had no active change record. Downstream product tests/coverage were skipped. This draft corrects missing scope documentation; it does not fabricate approval or imply CI is green. The exact new committed metadata range still needs coordinator Trust/provenance checks.

The Linux-only non-UTF8 filename test and Windows-only junction test are byte-preserved but were not compiled/executed on macOS. Leif directly approved the prepared scope on 2026-10-16 and the native writer recorded actor 0xLeif. Current qualification, actual independent implementation review and supported finalization remain separate gates. Local child CI/color variables were unset only for local test fixtures; no CI configuration changed.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
