---
id: finish-the-same-actor-scoped-review-user-facing-copy-so-cli-adopting-and-generated-agent-skills-no-longer-demand-a
state: archived
type: feature
base_commit: 8e658a1f122df2af13edea8a4be7f4a3365e4a9d
---

# Finish the same-actor scoped-review user-facing copy so CLI, ADOPTING, and generated agent skills no longer demand a second identity

## Intent

Finish the same-actor scoped-review user-facing copy so CLI, ADOPTING, and generated agent skills no longer demand a second identity

## Affected Canonical Specs

- `cmd_change`
- `agents`

## Acceptance Criteria

- CLI next-action and ship-status copy uses --reviewer <human> and names scoped review, not a second identity.
- docs/ADOPTING.md no longer tells adopters to pass --reviewer "<someone else>".
- Tracked Claude, Codex, Cursor, and Gemini SKILL.md files match the current generated skill body: the reviewer MAY be the definition approver.

## No-spec Rationale

Not applicable
