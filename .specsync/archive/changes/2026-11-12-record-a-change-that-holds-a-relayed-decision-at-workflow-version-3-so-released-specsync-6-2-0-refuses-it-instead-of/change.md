---
id: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
state: archived
type: bug_fix
base_commit: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
---

# Record a change that holds a relayed decision at workflow version 3 so released SpecSync 6.2.0 refuses it instead of dropping relayed_by

## Intent

Record a change that holds a relayed decision at workflow version 3 so released SpecSync 6.2.0 refuses it instead of dropping relayed_by

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A change that holds a relayed approval, review or correction is recorded at workflow version 3 (origin 2, every digest unchanged), so the real SpecSync 6.2.0 release binary refuses it with 'was written by a newer SpecSync (workflow version 3); upgrade specsync to read it' and changes nothing, as REQ-change-158 states

## No-spec Rationale

Not applicable
