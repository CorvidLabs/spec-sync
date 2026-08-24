---
id: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
state: archived
type: feature
base_commit: ffbcf524b4847c5cebbed107975849b6427af324
---

# Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize

## Intent

Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize

## Affected Canonical Specs

- `change`
- `cmd_change`
- `generator`

## Acceptance Criteria

- change new names each affected module context.md with its substantive line count, counting only authored prose and never a generated scaffold
- a FAILED change check names where the lesson goes, including for a bare check with no id; a passing check prints nothing
- finalize writes lesson-bundle.md into the archive durably and next_action names the fold-back targets before the merge
- lessons policy lives in src/change.rs and the generator owns what a scaffold looks like; the command layer only renders
- frontmatter stripping has one definition matching view::strip_frontmatter, pinned by a discriminating multi-rule regression test

## No-spec Rationale

Not applicable
