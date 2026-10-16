---
change: extract-issues-command-integration-tests-without-changing-their-behavior
artifact: testing
---

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
