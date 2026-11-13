---
change: errors-about-a-change-s-lifecycle-files-name-the-file-and-a-newer-specsync-s-file-is-refused-alike-everywhere
artifact: plan
---

# Plan

## Approach

Two shared helpers, used by every lifecycle loader and by the audit.

## Out of scope

`doctor`'s own reporting of a newer schema; lane A owns doctor and was told about it.

## Steps

1. Red integration tests (`tests/integration/lifecycle_file_errors.rs`).
2. `read_lifecycle_json` in the loaders.
3. `ensure_lifecycle_files_supported` in `load_change` and the audit.
4. Update any test that pins an old message; run targeted tests, fmt and clippy.

## Risks

Messages change shape, so tests that pin the old text need updating. Every load now scans the
change's file headers: one small read per file, on directories that hold at most about a dozen
files.

## Constraints consulted

#1044: members a newer SpecSync wrote are kept on rewrite. A newer schema is now refused before any
rewrite, which is stricter and consistent with that rule.
