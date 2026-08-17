---
change: CHG-0141-a-directory-named-in-files-must-score-zero-not-eighty
artifact: requirements
---

# Requirements

Two requirements as semantic deltas, in the two modules whose contracts change.

## `deltas/exports.md` — REQ-exports-010

The classification rule: a directory is its own outcome, decided before any read, and never
reported as unreadable.

Its last criterion — that the directory predicate is shared by every command asking the
question — is the one that matters. Without it a directory can be classified one way by `check`
and another by `score`, which is the disagreement this change exists to remove.

## `deltas/scoring.md` — REQ-scoring-005

The scoring rule: zero and grade F, naming the directory, rather than the 80 that came from
scoring freshness 15/15 on a path that merely exists.

Kept separate from the classification requirement because they can fail independently: the scan
could classify correctly while scoring still credits the path for existing.

## Declared modules without deltas

Seven of the nine declared spec modules — validator, cli_args, mcp, cmd_diff, cmd_score,
cmd_issues, cmd_lifecycle — consume the new classification without changing their own contracts.
They are declared because their source changed and the lifecycle requires declared scope to
match, not because they gained requirements.

## Explicitly retained behaviour

- `check` continues to hard-fail a directory mapping, unchanged.
- A spec naming a real source file scores exactly as before.
- `score` remains a metric, so `--explain` and JSON still render for the affected spec.

The second is the vacuity control. Without it, scoring everything zero satisfies the headline.
