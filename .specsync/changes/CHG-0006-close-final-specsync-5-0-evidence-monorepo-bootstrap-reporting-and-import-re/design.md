---
change: CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re
artifact: design
---

# Design

The change strengthens existing trust boundaries with a backward-readable additive acceptance-evidence field:

- accepted and archive transitions validate passed evidence, current contract/workspace inputs, and change-specific delivery state;
- Git policy lookup and changed paths are normalized relative to the requested project root;
- canonical spec paths are protected while portable lifecycle workspaces remain ignored;
- adoption creates explicit bootstrap coverage and init avoids enabling Git-dependent coverage outside Git;
- no-spec declarations cannot contradict a public-contract interview answer;
- the single schema-v1 self-adoption record is recognized by its exact identity and rationale as a narrow migration exception;
- comment collection permits an empty canonical spec set and still renders SDD failures;
- OpenSpec and Spec Kit import traversal rejects symlinks before reading or copying.

Regressions exercise local, CI, subproject, non-Git, overlapping-change, and symlink cases.
