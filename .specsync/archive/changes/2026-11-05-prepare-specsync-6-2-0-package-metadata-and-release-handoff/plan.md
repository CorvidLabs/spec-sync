---
change: prepare-specsync-6-2-0-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Synchronize only release metadata and document a bounded candidate from the selected source, mirroring the 6.1.2 preparation (8873c08d).

## Out of scope

No runtime fixes or dependency upgrades. No tag creation or movement. No push, merge, publication or deployment. No advance of the Action omitted-input default.

## Steps

Inspect current release workflows and versions; bump Cargo and the self-built CI and Trust candidate inputs; update the release validator fixtures; prepare CHANGELOG and the docs/releases/6.2.0.md handoff from the PRs since v6.1.2; refresh current-install guidance to published 6.1.2; run the three release validators, the pre-push gate and the unit tests.

## Risks

A proposed version must not imply publication or owner acceptance. Three included PRs are unmerged; if any is dropped, its CHANGELOG and handoff entries must be removed before tagging. Platform qualification remains a hard release dependency.

## Constraints consulted

Read repository AGENTS.md, docs/RELEASING.md section 1, native SDD policy, the archived 6.1.2 preparation change and current release workflows. Preserve the configured Trust policy and actual reviewer identity.
