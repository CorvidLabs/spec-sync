---
change: CHG-0049-document-the-verified-lifecycle-semantic-delta-format-and-surface-the-artifact-c
artifact: research
---

# Research

The documentation claims are grounded in these repository sources:

- `src/change.rs`: delta parsing, exact affected-module validation, permanent requirement tombstones, artifact validation, requirement-evidence discovery, effective-contract composition, dependency ordering, prepared canonical writes, and duplicate-application errors.
- `examples/sdd-lifecycle/run.sh` and `examples/sdd-five-epics/run.sh`: end-to-end creation, definition approval, implementation, verification, acceptance, and archival.
- `examples/sdd-concurrent-changes/`: declared dependencies and ordered effective-contract composition.
- Hosted checks for PRs #390 and #391: both patches passed their site-specific work but strict validation rejected their unowned meaningful paths.

Review identified and corrected two claims from the original reference draft: requirement evidence is discovered through exact IDs in `testing.md` or detected test files, while configured verification commands are a separate passing condition; active deltas are topologically ordered by declared dependencies with deterministic change-ID ordering among otherwise independent changes, not by delivery base commit.
