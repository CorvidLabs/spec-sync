---
change: commit-identities-from-change-records-are-used-only-as-full-commit-ids
artifact: design
---

# Design

Commit identities that SpecSync reads from change records are persisted workspace values, so they
are validated before use (REQ-change-013). Each place that passes one to Git as a revision now
requires a full commit id (`is_canonical_commit_id`), and the revision follows `--end-of-options`,
as `specsync diff` already does.

- The coverage diff base (`recorded_diff_base`): a recorded base that is not a commit id falls back
  to the base used when no change records one (`HEAD~1...HEAD`, or `HEAD`).
- `--end-of-options` goes before the base in both coverage diffs, `comparison_base_commit`'s
  merge-base, and the policy read at the comparison base.
- A full commit id is required for the semantic-succession base, the succession-base worktree (which
  also takes `--end-of-options`), shallow-history completeness, bootstrap records, verification and
  scoped-review commits, the reopening anchor, and status evidence.
- `--` goes before the policy path in the versioned-SDD probe.

A value that is not a commit id is treated as absent (or as not an ancestor), never passed on.
