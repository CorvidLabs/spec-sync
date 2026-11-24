---
change: prepare-specsync-6-4-2-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Prepare the next patch using the existing SpecSync release runbook. Reconcile changelog entries and examples with the merged source and validate them through the same release scripts that CI uses.

## Out of scope

Stable promotion, registry publication and moving the floating Action tag are subsequent release operations with their own evidence.

## Steps

1. Refresh from main after the selected fixes merge and confirm the dependency audit passes.
2. Set the patch version, explicit consumer pins and examples, and update the release-candidate test fixtures.
3. Reconcile the changelog with the previous release tag and document candidate qualification in the handoff.
4. Run the release validators, local verification lane, strict spec and change checks, pre-push and Trust.
5. Complete scoped review and archival in this PR and obtain green hosted checks before selecting the exact main SHA for a candidate.

## Risks

Publishing pin examples too early can direct users to an unavailable binary. Retain published installation guidance and the wrapper default until stable assets exist. A late merge can change the release contents. Recheck the diff and evidence at the final revision.

## Constraints consulted

Use docs/RELEASING.md, docs/ci-confidence.md and the 6.4.1 release handoff. The release tag must identify a commit on origin/main whose Cargo version matches it.
