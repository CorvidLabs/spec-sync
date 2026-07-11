---
change: CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation
artifact: context
---

# Context

The initial 5.0 candidate is feature-complete and cross-platform CI is green, but automated review found safety paths not represented in the original suite. This hardening change treats canonical-data loss, stale evidence, fail-open policy, unordered deltas, and unverifiable path coverage as release blockers. Proof now includes a packaged clean consumer, a real composite-Action consumer gate, 4.x/OpenSpec/Spec Kit adoption fixtures, executable lifecycle examples, and a clean five-epic product evolution with a preserved 16-commit audit trail. That simulation exposed and now covers a post-merge archive edge case: a successful Git comparison with empty output is valid evidence, not an unavailable comparison. Native skill files install for all four supported agents; live model discovery remains explicit because the environment security reviewer blocks transmission of project-local skill metadata to remote agent services.
