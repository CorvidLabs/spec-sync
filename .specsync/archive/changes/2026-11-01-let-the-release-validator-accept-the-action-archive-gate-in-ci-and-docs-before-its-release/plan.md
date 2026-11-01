---
change: let-the-release-validator-accept-the-action-archive-gate-in-ci-and-docs-before-its-release
artifact: plan
---

# Plan

## Approach

Select the packaged consumer step by its `version` input. Allow `@main` only on documented steps
that set `require-archived` or `archive-gate-only`, and only while `PUBLISHED_ACTION_DEFAULT` is
at or below `ARCHIVE_GATE_UNRELEASED_AFTER` (6.1.1).

## Out of scope

Release promotion, the Action default, and every other documented step's pinning rule.

## Steps

1. `validate-release-version.py`: parse an `archive_gate` flag per documented step, add
   `ARCHIVE_GATE_UNRELEASED_AFTER` and `release_key`, and filter the consumer lookup by `version`.
2. `test-validate-release-candidate.py`: prove the allowance expires at a newer published
   release, and that `@main` without archive-gate inputs is still rejected.

## Risks

The allowance outliving its purpose. Mitigated because it ends when the published default advances.

## Constraints consulted

none
