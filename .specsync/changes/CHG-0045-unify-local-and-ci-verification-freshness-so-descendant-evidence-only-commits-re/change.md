---
id: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
state: verifying
type: bug_fix
base_commit: b4a90aa2871cb77cd824df759e3e90a31ab3e971
---

# Unify local and CI verification freshness so descendant evidence-only commits remain current while source, test, configuration, contract, or nonancestor changes fail closed

## Intent

Unify local and CI verification freshness so descendant evidence-only commits remain current while source, test, configuration, contract, or nonancestor changes fail closed

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Local and CI use one freshness predicate; one or multiple descendant commits touching only excluded lifecycle-evidence paths remain current; source, test, configuration, contract, or nonancestor changes fail closed; contract and workspace digests always match; summaries and strict checks agree; and canonical REQ-change-013 and REQ-change-016 plus regressions document the behavior.

## No-spec Rationale

Not applicable
