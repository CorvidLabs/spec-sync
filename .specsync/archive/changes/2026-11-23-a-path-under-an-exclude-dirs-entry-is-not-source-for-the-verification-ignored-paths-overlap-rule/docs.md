---
change: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule
artifact: docs
---

# Docs

`site/src/content/docs/configuration.md`: the `verification_ignored_paths` section says a path under an `exclude_dirs` name below a `source_dirs` root is not source, with the NFTMix example, and that only the `meaningful_paths = ["."]` half of "a project whose `source_dirs` or `meaningful_paths` is `.` cannot use the list" still holds. CHANGELOG: an Unreleased Fixed line.
