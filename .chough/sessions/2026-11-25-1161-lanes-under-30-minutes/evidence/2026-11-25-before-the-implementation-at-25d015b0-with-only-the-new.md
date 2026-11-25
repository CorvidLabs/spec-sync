---
kind: evidence
id: 2026-11-25-before-the-implementation-at-25d015b0-with-only-the-new
aliases: []
claim: "Before the implementation, at 25d015b0 with only the new contract tests added: 61 of 98 tests in test-validate-release-candidate.py failed (the validator had no parts or part-record mode, the slice script and the rehearsal workflow did not exist, and the qualify job was one job per platform at 120 minutes), and 4 of 6 in test-workflow-time-limits.py failed, naming the jobs over 30 minutes or with no limit."
result: failed
evidence_kind: product-test
revision: a29b4a439c9f19b5103d30b5c40e4c32c010d678
content: 253180fc8d09f289ae64634152d74aece54980851c7599e69f736ab6f8039bed
environment: macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two tests at once
fingerprint: sha256:a6756651457a18b7f76d0584ade602df7a15ae07a45d37f889c84ae331d3e4ce
external: ext-22f4de
criteria: [SPEED-2.b.4]
requirements: []
contracts: []
limitations: []
observed: 2026-11-25T04:33:22Z
actor:
  - agent: claude-code
    terminal: t14v
---
**Before the implementation, at 25d015b0 with only the new contract tests added: 61 of 98 tests in test-validate-release-candidate.py failed (the validator had no parts or part-record mode, the slice script and the rehearsal workflow did not exist, and the qualify job was one job per platform at 120 minutes), and 4 of 6 in test-workflow-time-limits.py failed, naming the jobs over 30 minutes or with no limit.**

Recorded as failed in macOS 27.2 (Darwin 27.2.0), Apple M5 Ultra, local clone, one Cargo job and two tests at once at revision a29b4a439c9f, observed 2026-11-25T04:33:22Z.

## Key failing lines

- FAILED (failures=25, errors=36)
