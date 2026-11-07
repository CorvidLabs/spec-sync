## ADDED

### REQUIREMENT REQ-cmd-merge-004

When the configured specs directory is the repository root, `specsync merge --install` SHALL emit attribute patterns that Git matches for root and nested contracts.

Acceptance Criteria
- `specs_dir = "."` and `specs_dir = "./"` produce rules for `*.spec.md` and `requirements.md` without a `./` directory prefix.
- `git check-attr merge` reports `merge: specsync` for root and nested `.spec.md` and `requirements.md` paths.
- Existing non-root directory normalization, wildcard escaping, local driver configuration, symlink refusal, and failure rollback remain unchanged.
