---
change: CHG-0049-document-the-verified-lifecycle-semantic-delta-format-and-surface-the-artifact-c
artifact: docs
---

# Docs

Add `site/src/content/docs/deltas.md` as the focused reference for:

- semantic-delta locations and exact module coverage;
- `ADDED`, `MODIFIED`, and `REMOVED` requirement and spec-section blocks;
- requirement evidence and permanent removal tombstones;
- effective-contract composition, conflicts, and dependency ordering; and
- atomic canonical application at acceptance.

Update `site/src/content/docs/quickstart.md` to name the artifact-completeness and exact delta-module gates before the first lifecycle approval.

The consolidated PR supersedes PR #391 only after its quickstart commit is safely present on PR #390. No canonical spec, public API, or release-version documentation changes are included.
