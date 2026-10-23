---
change: authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage
artifact: testing
---

# Testing

Positive regression: create a meaningful policy migration, obtain explicit test-fixture approval/review through the native fixture helpers, verify/finalize/commit its archive on the same branch, then run active audit against the original policy base with no active records. Existing6.1.1 must fail this discriminator; repaired code must pass.

Negative controls: absent migration, wrong path, unrelated delivery archive, malformed or tampered immutable evidence, policy bytes changed after acceptance, and attempted policy disable without authenticated coverage must fail. A damaged unrelated archive must not make active audit walk all history. Preserve existing active-record behavior and archive-integrity tests.

Required verification includes targeted change tests, full fledge lanes run verify and fledge trust verify, strict canonical/scoped verification, Clippy and hosted product/Trust gates according to AGENTS. Actual Algonest48 reproduction and exact source/artifact/digest provenance are separate evidence. Nothing in this draft claims those future checks passed.


## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-change-110 | Nine `archived_policy_audit_` regressions in `src/change_tests/substantive_plan.rs` pass; the original implementation fails the genuine finalized-policy positive discriminator. Candidate passes the unchanged exact-head Algonest48 lifecycle guard, while official6.1.1 fails its policy-coverage check on the same checkout. Full product lane, strict scoped/canonical checks, Trust and exact-head hosted results remain required before delivery and are not claimed by the focused tests. |


## Observed native results (2026-10-23)

All required native verification steps pass on the frozen source: formatting, Clippy and types; 2,600 unit tests and464 integration tests; optimized release build;62 strict specs with100% file/LOC coverage;64 release-workflow validator tests and29 required-CI-gate tests. The nine focused audit regressions pass, the original implementation fails the positive discriminator, and the candidate passes the unchanged exact-head Algonest48 lifecycle gate.

The first full run exposed a temporary-path Unix-socket length limit, corrected with a shorter workspace TMPDIR and a full test rerun. The workflow validators then exposed missing Ruby/Psych; signed official Fedora RPMs were extracted into task-local storage, Psych/JSON smoke checks passed, and Fledge resumed the failed and remaining validator steps. No source expectation, test or gate was skipped or relaxed. One inherited unused_mut warning remains in existing test code. Native Trust, provenance and exact-head hosted checks remain separate delivery gates.
