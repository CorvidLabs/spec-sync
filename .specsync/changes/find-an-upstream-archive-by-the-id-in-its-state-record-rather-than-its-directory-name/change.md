---
id: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name
state: draft
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Find an upstream archive by the id in its state record rather than its directory name

## Intent

Find an upstream archive by the id in its state record rather than its directory name

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- `upstream_archive_of` finds the remote default's archive of a change by the `id` in each package's `state.json`, as `find_change_dir` does, so a renamed or undated package is found like a `YYYY-MM-DD-<id>` one.
- The lookup stays bounded: one `git ls-tree -r` of the remote default's archive directory and one `git cat-file --batch` read of the `state.json` blobs, then the adding commit and one ancestry check only for a matching package. It never rebuilds the full archive index, and the #861 lifecycle spawn-ceiling test still passes.
- A unit test with a renamed upstream package fails on main first and passes after. The existing upstream-archive tests stay green.
- Every other behavior of the upstream-archive refusal is unchanged.

## No-spec Rationale

Not applicable
