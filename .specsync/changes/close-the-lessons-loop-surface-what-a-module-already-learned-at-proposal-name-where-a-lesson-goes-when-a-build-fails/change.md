---
id: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
state: implementing
type: feature
base_commit: e7fe36f0aacf33452ca5243741ceb7490ea336cc
---

# Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize

## Intent

Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change new names each affected module context.md with its substantive line count, and stays silent for a module whose context holds only scaffold
- a FAILED change check names where the lesson goes; a passing check prints nothing
- finalize writes lesson-bundle.md into the archive and next_action names the fold-back targets before the merge
- lessons policy lives in src/change.rs and the command layer only renders it
- frontmatter stripping has one definition inside the change module, matching view::strip_frontmatter, pinned by a horizontal-rule regression test

## No-spec Rationale

Not applicable
