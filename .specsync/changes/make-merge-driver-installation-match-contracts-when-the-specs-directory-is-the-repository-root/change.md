---
id: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
state: draft
type: bug_fix
base_commit: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
---

# Make merge driver installation match contracts when the specs directory is the repository root

## Intent

Make merge driver installation match contracts when the specs directory is the repository root

## Affected Canonical Specs

- `cmd_merge`

## Acceptance Criteria

- For specs_dir = . and ./, merge --install writes rules that git check-attr resolves as merge=specsync for root and nested .spec.md and requirements.md paths.
- The git check-attr regression test fails on the unmodified base before the fix and passes afterward; existing non-root install, quoting, symlink refusal, and rollback behavior stays unchanged.
- The change traces to MERGE-1 without new HI wording and adds one cmd_merge requirement assigned by the orc.

## No-spec Rationale

Not applicable
