---
kind: evidence
id: 2026-11-24-1161-contract-suites-pass-on-the-edit
aliases: []
claim: "With timeout-minutes for the qualify job at 120: validate-release-version and validate-workflow-runtime-pins pass, test-validate-release-candidate passes 67 tests, test-required-ci-gate 38, test-linux-baseline 35, test-ci-trigger-paths 3, test-ci-attest-and-summary 9, test-clippy-all-targets 5, test-classify-ci-paths passes, and actionlint reports nothing. Parsed as YAML the workflow is identical to main apart from that one value. No product, test or spec file is changed, and no test is dropped or weakened."
result: passed
evidence_kind: structural
method: The release validators and CI contract suites run directly on the edited tree; the workflow parsed as YAML before and after
revision: 1dfd0dbf5cd9eed09cfbc5ef49b3351cc696646c
content: 0c8817fa7b1f6660deddd16db860c1c3dfd48732718cba644034a7f93597f5a5
environment: macos-arm64-local
criteria: [SPEED-2.b.2]
requirements: []
contracts: []
limitations:
  - "Structural only: whether 120 minutes is enough is shown by the v6.4.2-rc.2 qualification, not here"
observed: 2026-11-24T18:43:23Z
actor:
  - agent: claude-code
    terminal: t14v
---
**With timeout-minutes for the qualify job at 120: validate-release-version and validate-workflow-runtime-pins pass, test-validate-release-candidate passes 67 tests, test-required-ci-gate 38, test-linux-baseline 35, test-ci-trigger-paths 3, test-ci-attest-and-summary 9, test-clippy-all-targets 5, test-classify-ci-paths passes, and actionlint reports nothing. Parsed as YAML the workflow is identical to main apart from that one value. No product, test or spec file is changed, and no test is dropped or weakened.**

Recorded as passed in macos-arm64-local at revision 1dfd0dbf5cd9, observed 2026-11-24T18:43:23Z.
