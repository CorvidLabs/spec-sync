---
id: ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so
state: implementing
type: bug_fix
base_commit: fb88b9acaafe99abd83a637876331e83330e49fb
---

# Ship must name the lesson fold-back too: the archive bundle is written and only finalize says so

## Intent

Ship must name the lesson fold-back too: the archive bundle is written and only finalize says so

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- change ship names the lesson fold-back targets and the bundle path before the merge step, exactly as change finalize already does
- a change owning no specs gets guidance byte-identical to before the fix, so the prefix cannot leak into cases with nothing to fold
- the sibling do-not-merge blocker survives the fold-back prefix rather than being displaced by it
- the shared guidance is one pure function pinned by tests, not a string duplicated per verb

## No-spec Rationale

Not applicable
