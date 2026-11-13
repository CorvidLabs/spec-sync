---
kind: bug
id: 2026-11-07-merge-install-ignores-a-root-specs-directory-871
aliases: [BUG-merge-install-ignores-a-root-specs-directory-871-408287a7fe659cca53621579]
title: Merge install ignores a root specs directory (#871)
status: verified
classification: product-defect
scope: blocking
impact: Projects whose specs directory is the repository root get no merge-driver protection while installation claims success.
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
cases: []
evidence:
  - OBS-product-test-b47ebf5a0d626500e6c08d9c
  - OBS-product-test-8b3cf28186053c41615bf658
  - OBS-product-test-e62f4a4b773df570bf80de09
receipts: []
fixture: target/tmp/ss-871-repro
environment: local-macos-arm64
revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
found_by: "GitHub issue #871 and local reproduction at b3ca5f97"
related: []
created: 2026-11-07T03:12:35Z
updated: 2026-11-07T15:15:54Z
---
# Merge install ignores a root specs directory (#871)

**What happened:** merge --install reports success but git never applies the SpecSync driver to contracts
Actual: The installer writes ./** patterns and git check-attr reports merge: unspecified for all four paths

**Steps:** Trigger: Configure specs_dir = . and install the merge driver
1. Create a git repository with .specsync/config.toml containing specs_dir = .
2. Run specsync merge --install
3. Run git check-attr merge for root and nested contract paths

**Expected:** Root and nested .spec.md and requirements.md paths have merge=specsync

**Why it matters:** Projects whose specs directory is the repository root get no merge-driver protection while installation claims success.

**Fix:** Recorded before 0.6.0; the Log keeps its history.

## Log

- 2026-11-07T03:12:35Z · Reported (GitHub issue #871 and local reproduction at b3ca5f97); evidence: OBS-product-test-b47ebf5a0d626500e6c08d9c
- 2026-11-07T15:15:54Z · status: open -> verified. Verified after explicit bare-dot normalization: 6/6 install tests pass and the original merge --install plus git check-attr scenario selects specsync for all four root/nested contract paths.; evidence: OBS-product-test-8b3cf28186053c41615bf658, OBS-product-test-e62f4a4b773df570bf80de09
