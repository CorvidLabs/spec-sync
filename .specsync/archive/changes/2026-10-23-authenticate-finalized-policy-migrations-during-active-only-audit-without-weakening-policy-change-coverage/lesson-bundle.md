# Lesson bundle — authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Authenticate finalized policy migrations during active-only audit without weakening policy-change coverage
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/validate_loaded_change.rs, src/change_tests/substantive_plan.rs, specs/change
- **Acceptance**: A valid finalized same-delivery policy migration remains recognized by active-only audit after all workspaces archive. Only authenticate relevant archive candidates and require their signed policy entry to match current policy bytes. Reject no-record, wrong-scope, forged/tampered, unrelated historical archive and later unsigned policy edits. Preserve active audit scope, strict enforcement, canonical coverage, existing approvals and every CI gate. Prove the original Algonest archived-policy failure and repair with regressions and negative controls. Submit upstream repair without merging, releasing or deploying.

## Evidence

- Verification commit: `66f58cbe7a32928666557f2e139cf15cd3de6916`
- Base commit: `825c93e74663452e1927b59acee7fa0fed9ffc9c`
- Verified by: `specsync check --spec change --strict`

## From the change's context.md

# Context

Algonest #48 is a same-PR finalized policy adoption, head72465f1. Official SpecSync6.1.1 audit rejects it because check_project_with_command_output checks only active records for policy coverage. The immutable package explicitly covers .specsync/sdd.json, but records is empty after finalization. Algonest #49's audit passes against #48's stacked base; this does not repair #48 versus main. The separate missing-Git-object error on #49 was resolved by fetching authentic history and is not part of this source repair.

Read specs/change/change.3md: active audits must remain focused on living records, archives must be authenticated selectively, and a same-PR archive must retain its exact delivery obligations. Do not restore or edit old approvals or workflow baseline. Current release is6.1.1; this work proposes an upstream PR, not a release or merge.

## From the change's design.md

# Design

The boundary is the living policy comparison inside check_project_with_command_output. Active records retain their existing path. A fallback may use archived_records_in_delivery only for packages actually in that diff, then authenticate exact policy ownership and accepted bytes. The helper returns coverage or a diagnostic, never a boolean based solely on affected_paths. No public command, persisted schema, engine version or normal archive roster count changes.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
