---
id: CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be
state: archived
type: bug_fix
base_commit: 2991eb272d1299e5db4b96cc76df3e9c1b1a9b86
---

# A semantic delta section body may contain subheadings so scaffolded specs can be changed

## Intent

A semantic delta section body may contain subheadings so scaffolded specs can be changed

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A semantic delta whose spec-section body contains subheadings is accepted, so the Public API and Dependencies sections a scaffold generates can be changed through the lifecycle without hand-editing the spec first. A subheading appearing before any delta item is still rejected, and the message names the two valid item forms. Existing projects benefit without regenerating their specs.

## No-spec Rationale

Not applicable
