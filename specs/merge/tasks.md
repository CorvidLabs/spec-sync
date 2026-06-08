---
spec: merge.spec.md
---

## Tasks

(none open)

## Done

- [x] `merge_specs` driver: git-conflicted vs all-files scan, dry-run, write-back
- [x] `has_conflict_markers` detection of `<<<<<<< ` markers
- [x] `detect_conflicted_specs` via `git diff --name-only --diff-filter=U`
- [x] `parse_conflict_regions` splitting content into clean text and ours/theirs conflict blocks
- [x] `detect_section` context detection (last `## ` heading) to choose a strategy
- [x] Frontmatter conflict resolution: list union+sort, scalar "theirs wins"
- [x] Changelog conflict resolution: chronological merge, dedup by full row
- [x] Generic table conflict resolution: dedup by first cell, "theirs wins"
- [x] Prose conflicts left as `Manual` with markers preserved
- [x] Post-resolution frontmatter validation warning
- [x] `print_results` (colored text) and `results_to_json` output formats
- [x] Custom zero-dep YAML field parser (`parse_yaml_fields`)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
