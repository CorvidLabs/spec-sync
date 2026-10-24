---
change: prepare-specsync-6-1-2-package-metadata-and-release-handoff
artifact: plan
---

# Plan

## Approach

Synchronize only release metadata and document a bounded candidate from the selected source.

## Out of scope

No runtime fixes or dependency upgrades. No tag creation or movement. No merge or publication or deployment.

## Steps

Inspect current release workflows and versions; prepare metadata and notes; run native checks; open a draft PR with every unmet gate explicit.

## Risks

A proposed version must not imply publication or owner acceptance. Pending security work and platform qualification remain hard release dependencies.

## Constraints consulted

Read repository AGENTS.md and fledge.toml and native SDD policy and current release workflows. Preserve the configured Trust policy and actual reviewer identity.
