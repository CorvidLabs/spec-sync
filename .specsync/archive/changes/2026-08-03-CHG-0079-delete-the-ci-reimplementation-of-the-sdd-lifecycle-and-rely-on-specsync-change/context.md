---
change: CHG-0079-delete-the-ci-reimplementation-of-the-sdd-lifecycle-and-rely-on-specsync-change
artifact: context
---

# Context

This repository contains two implementations of the SDD lifecycle.

`src/change.rs` and `src/archive.rs` are 29,337 lines of Rust, covered by `cargo test`, and shipped
to every SpecSync user. `.github/scripts` and the lifecycle workflows are roughly 7,257 lines of
Python, bash, and YAML that re-derive the same rules from Git commit topology. The second one is not
shipped, is tested only by bespoke harnesses, and has drifted from the first.

Two defects show the drift concretely. `reuse-check-from-ancestors.py` does not recognize the
finalization edge, so PR #494 could not validate the very sequence it introduced.
`post-merge-archive.yml` used a greedy `.+` capture and counted a change package's `deltas/`
subdirectory as a second archive root, so #494's archive was never bound to its merge commit.

Both defects are in the copy, not in SpecSync.

The copy exists to support one self-imposed constraint: lifecycle metadata must live in commits
separate from the product change, and each such commit's checks must independently pass. That
constraint is the sole reason `reuse-check-from-ancestors.py` exists at 2,522 lines plus 1,610 lines
of tests. `ci.yml` encoded the constraint directly — a green implementation PR was failed on purpose
with "Run specsync change finalize, commit the exact archive move, and push it before GitHub merge."

The constraint also grew the protected surface faster than the project could maintain it.
`lifecycle-policy-guard.yml` fail-closed on every protected path with no green path for any pull
request that touched one, so #491, #492, and #494 were each merged red by administrator bypass.
