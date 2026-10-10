---
id: make-it-obvious-on-a-pull-request-when-a-change-is-still-open-and-when-it-is-archived
state: implementing
type: feature
base_commit: ee89640c3ccd3c68d044ff0a58499749b2fba92c
---

# Make it obvious on a pull request when a change is still open and when it is archived

## Intent

Make it obvious on a pull request when a change is still open and when it is archived

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- When I look at a pull request, I can tell whether its change is still open or already archived.
- A pull request that still has an open change does not look finished.
- A pull request whose changes are all archived says it is ready to merge once the checks are green.
- When it is time to archive, one command archives the change, commits that archive, and tells me that merging is the step that is left.
- When it is not time, that same place tells me the one reason and the one command.
- When more than one change is open, I am told which one to archive first.

## No-spec Rationale

Not applicable
