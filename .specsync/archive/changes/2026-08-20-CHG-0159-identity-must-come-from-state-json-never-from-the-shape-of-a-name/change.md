---
id: CHG-0159-identity-must-come-from-state-json-never-from-the-shape-of-a-name
state: archived
type: bug_fix
base_commit: 0387678fbbbd9b37ec9db6f94ddb503878b7d3f6
---

# Identity must come from state.json, never from the shape of a name

## Intent

identity must come from state.json, never from the shape of a name

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Two gates currently decide identity from the shape of a name rather than from state.json, and both fail open. (1) is_positive_legacy_tombstone treats a directory as a real lifecycle package only if its name contains the literal '-CHG-'. An undated package named CHG-0001-foo does not contain that substring, so a real archived change that lost its lifecycle files is silently skipped by list_all_changes_uncached and located_change_sequences instead of being refused as corrupt. This is live today, not hypothetical. (2) classify-ci-paths.sh decides which pull requests need the mandatory independent review by globbing .specsync/changes/CHG-*/state.json, and parses archive directory names with a regex requiring a date prefix and a CHG-NNNN ordinal. ci.yml gates the review job on that count, so any identity shape the glob does not match means review_required=false and the one mandatory human review silently stops running while CI goes green faster. Done when: a package is classified by what it contains rather than by what it is called, so a directory holding any regular file outside deltas is a lifecycle package and is refused when its state is unreadable; the CI classifier finds candidate changes and parses archive directories without assuming any ID shape; and both are pinned by tests that fail against the current binary.

## No-spec Rationale

Not applicable
