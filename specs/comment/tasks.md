---
spec: comment.spec.md
---

## Tasks

(none open)

## Done

- [x] `render_check_comment`: header, summary table, grouped errors/warnings, action items, unspecced files, footer
- [x] `spec_link` GitHub URL vs inline-code fallback
- [x] `suggestion_for_error` classification (missing section, source file, DB table, frontmatter, dependency, schema column, stale file ref, generic fallback)
- [x] `suggestion_for_warning` classification (export, consumed-by, schema column, generic fallback)
- [x] `group_by_spec` / `split_spec_prefix` / `strip_spec_prefix` message grouping and prefix handling
- [x] `detect_branch` via `git rev-parse --abbrev-ref HEAD`
- [x] Unspecced-files truncation at 15 entries
- [x] Unified output pipeline shared by the marketplace action and project CI
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)
- [x] Bound complete rendered comments to 49,152 bytes with UTF-8-safe truncation
- [x] Add explicit local reproduction guidance to truncated comments
- [x] Add focused oversized-Unicode renderer coverage

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
