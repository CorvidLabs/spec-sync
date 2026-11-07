---
spec: merge.spec.md
---

## Tasks

- [ ] Complete CHG-0066 full repository, strict spec, score, trust, and sandbox verification.
- [x] Complete fresh independent acceptance/adversarial review after the hardening fixes.
- [ ] Correct PR #448 metadata, obtain closing approval, merge, and archive CHG-0066.

## Done

- [x] Git merge driver for parallel module contracts (#862, MERGE-1): `merge_contract` counts both version bumps and unions change-log rows, `files:`, and requirement blocks. Marker resolution is unchanged.
- [x] A clean merge-driver result keeps both sides' meaning (MERGE-5, REQ-merge-006): YAML-typed list items stay quoted (#872), the change log ends at its first blank line (#875), version comments merge three ways (#876), renumbering stops at `u64::MAX` (#870).

- [x] `merge_specs` driver: git-conflicted vs all-files scan, dry-run, write-back
- [x] `has_conflict_markers` detection of `<<<<<<< ` markers
- [x] `detect_conflicted_specs` via `git diff --name-only --diff-filter=U`
- [x] `parse_conflict_regions` splitting content into clean text and ours/theirs conflict blocks
- [x] `detect_section` context detection (last `## ` heading) to choose a strategy
- [x] Frontmatter conflict resolution: known list union+sort, numeric version max, ambiguous scalars manual
- [x] Changelog conflict resolution: chronological merge, dedup by full row
- [x] Generic table conflict resolution: union distinct keys, deduplicate identical rows, ambiguous duplicate keys manual
- [x] Prose conflicts left as `Manual` with markers preserved
- [x] Post-resolution frontmatter validation blocks writes of invalid output
- [x] `print_results` (colored text) and `results_to_json` output formats
- [x] Custom zero-dep YAML field parser (`parse_yaml_fields`)
- [x] Populate requirements.md with user stories and acceptance criteria (2026-04-10)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
