---
id: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
state: implementing
type: bug_fix
base_commit: a1efb828b13fb5cca37aa6aec65b6677b334da2c
---

# Watch must disclose a dropped directory and not claim a pass over an empty check

## Intent

watch must disclose a dropped directory and not claim a pass over an empty check

## Affected Canonical Specs

- `watch`
- `cli`

## Acceptance Criteria

- watch reports every configured specs_dir or source_dirs entry that does not exist, naming the configured path and its role, in both human and JSON output, before it starts watching; watch reports a pass only when the check it ran examined at least one spec, and says nothing was checked otherwise; a missing directory stays non-fatal while at least one directory exists; an empty watch set still exits 1; check and its exit codes are unchanged; a real passing spec set still reports All checks passed

## No-spec Rationale

closes #577: watch silently dropped configured directories from its watch set and reported All checks passed over a check that examined no specs
