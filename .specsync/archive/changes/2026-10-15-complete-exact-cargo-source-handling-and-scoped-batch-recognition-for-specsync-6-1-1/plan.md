---
change: complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1
artifact: plan
---

# Plan

## Approach
Use shared safe source detection and leaf filename semantics; add bounded watch path/mode selection; correct documentation.

## Out of scope
Dependencies, new CLI options, project-root recursive scans, manifest globs, validator ownership redesign and protection changes.

## Steps
1. Record the real approved definition natively.
2. Implement non-overlapping scoped batch, source creation and watch corrections with focused causal tests.
3. Reconcile contracts and requirement evidence and independently review final diffs.
4. Run fresh full pinned verification, scoped materialization/check, Trust/Attest and pre-push; publish reviewable product and inspect exact-head CI.
5. Obtain genuine scoped implementation acceptance, archive in the same PR, verify archive CI, merge normally, qualify immutable RC on Ubuntu/macOS and publish through existing release surfaces.

## Risks
A divergent command helper can retain the exact-file bug; share selection. Overlapping file/directory roots can create false ambiguity; deduplicate. Following symlinks or scanning root parents can broaden scope; enforce confined no-follow admission. macOS backend probes may deliver no positive control; disclose this and require valid platform proof.

## Constraints consulted
AGENTS lifecycle and pre-push rules; generator no-overwrite/determinism lessons; existing exact-file ownership/security contracts; user approved bounded follow-up proposal.
