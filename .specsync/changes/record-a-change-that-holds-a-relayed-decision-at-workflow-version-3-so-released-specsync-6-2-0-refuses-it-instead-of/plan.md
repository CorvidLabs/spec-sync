---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: plan
---

# Plan

## Approach

Gate through `workflow_version`, the one field released 6.2.0 already refuses. Add version 3, the gated form of workflow v2, as design.md describes. Prove it with the real 6.2.0 binary.

## Out of scope

- #1064 part (b), the array-length gap in `collect_unknown_members`.
- Editing archived changes that already hold relays at version 2.
- The 6.3.0 version bump and release notes (the release-prep PR, task 2), apart from this change's CHANGELOG entry.

## Steps

1. Anchor, load and history rules for version 3; the `== 2` site becomes `>= 2`.
2. Execution digest reads 3 as 2.
3. Raise points in approve and review, the v1 note, and the `state.json` write helper.
4. Unit tests in `src/change_tests/` and integration tests in `tests/integration/relayed_decision.rs`.
5. `.github/scripts/test-relay-gate-compat.sh`: pinned 6.2.0 against records this build wrote. Run it as a fledge task and in the CI `closed-late-compat` job.
6. Spec: invariant 60, Change Log, version; a lesson plane in `change.3md`; docs in `workflow.md` and `README.md`; a CHANGELOG entry.

## Risks

- A comparison outside the digests may still read the raw version, for example an accepted-anchor equality. The full suite and the archive tests decide it; lesson z=34 says to project the same way everywhere.
- The history-check relaxation must not let a downgrade through; tests cover 3 to 2 and changed origins.

## Constraints consulted

- `specs/change/change.3md` z=34: compare a record the way its digest reads it.
- z=31: an old binary that cannot parse a record refuses the repository; here that refusal is the point.
- z=24: relays are claims, and no gate reads them.
