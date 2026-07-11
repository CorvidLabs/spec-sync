---
change: CHG-0014-route-archive-only-lifecycle-moves-through-a-minimal-specsync-ci-gate-while-pres
artifact: design
---

# Design

Add one first-party classifier job that reads the NUL-delimited Git diff and
emits `archive_only`, `full`, `site`, and `vscode` outputs.

An archive-only diff may contain:

- additions under `.specsync/archive/changes/**`
- deletions under `.specsync/changes/**`

If an active-workspace path still exists, or any unrelated path changed, the
diff is not archive-only. Unknown paths select full CI.

Every triggered run executes the classifier and `spec-check`. Heavy Rust,
audit, coverage, Action, site, and extension jobs use classifier outputs. A
stable `Required CI gate` accepts only successful or intentionally skipped
dependencies. Main attestation depends on that aggregate gate.
