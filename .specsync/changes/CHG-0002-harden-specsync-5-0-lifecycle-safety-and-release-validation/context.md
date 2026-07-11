---
change: CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation
artifact: context
---

# Context

The initial 5.0 candidate is feature-complete and cross-platform CI is green, but automated review found safety paths not represented in the original suite. This hardening change treats canonical-data loss, stale evidence, fail-open policy, unordered deltas, and unverifiable path coverage as release blockers. Proof now includes a packaged clean consumer, a real composite-Action consumer gate, 4.x/OpenSpec/Spec Kit adoption fixtures, and executable lifecycle examples. Native skill files install for all four supported agents; actual model discovery remains an explicit external-authentication check because the local Claude/Gemini sessions are not authenticated and exporting repository metadata to remote agent services requires user authorization.
