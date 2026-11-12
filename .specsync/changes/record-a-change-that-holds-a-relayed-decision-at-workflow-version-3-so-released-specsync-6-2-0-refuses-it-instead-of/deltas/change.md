---
module: change
---

## ADDED

### REQUIREMENT REQ-change-158

A change that holds a relayed decision SHALL be recorded at workflow version 3,
so that a released SpecSync that predates relayed decisions, such as 6.2.0,
refuses the change with its "written by a newer SpecSync" message instead of
rewriting it and dropping `relayed_by` (#1064). Workflow version 3 SHALL be
workflow v2 with that one difference: every workflow-v2 rule applies to it
unchanged, its immutable `workflow_origin_version` stays 2, and no digest
reads the difference.

#### Acceptance Criteria

- `change approve` with a relayed decision, including an accepted definition
  refresh, and `change review` with a relayed decision write the change's
  `state.json` at workflow version 3 before they write the relay, so an
  interrupted command never leaves a relayed decision on a version-2 change.
- Any other rewrite of a version-2 change's `state.json` raises it to 3 when
  its `approvals.json`, `review.json`, `review-attempts.json` or
  `corrections.json` holds a `relayed_by`. A change archived before this
  version is not edited, and keeps version 2 until such a rewrite.
- A change with no relayed decision stays at workflow version 2, and
  `change new` creates the same version it does today.
- A workflow-v1 change, which a `--portable-5-0-1` pair requires, keeps
  version 1, because raising it would change its workflow; a relayed decision
  recorded on one is not gated, and `change approve` and `change review` print
  a note saying that SpecSync 6.2.0 and older can still drop it.
- Workflow version 3 is valid only with origin 2. The workflow-version history
  check accepts one transition, 2 to 3 with the origin unchanged, between
  commits and between `HEAD` and the working tree. It still refuses 3 back to
  2, and every other change of version or origin.
- Raising the version changes no recorded digest: the execution digest reads
  version 3 as 2, so a verification recorded before a relayed review still
  binds, and so does every review, closing and finalization digest.
- Finalize keeps version 3 in the archived `state.json` and
  `accepted-state.json`, and this SpecSync reads an archived version-3 change
  as it reads an archived version-2 one. A version above 3 is still reported
  as written by a newer SpecSync (REQ-change-079).
- Run against a repository where this SpecSync recorded a relayed approval and
  a relayed review, the real SpecSync 6.2.0 release binary, pinned by sha256,
  exits non-zero on that change's lifecycle commands with "was written by a
  newer SpecSync (workflow version 3); upgrade specsync to read it", and every
  file under `.specsync/` is byte-identical afterwards. With that change
  archived, 6.2.0 `change new` and `change audit` refuse the same way and change
  nothing, and 6.2.0 `check --strict` still passes.
