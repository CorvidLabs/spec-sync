---
change: close-changes-that-will-not-ship
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-159

`change close` SHALL close a draft, approved, implementing or verifying change that will not
ship by moving its package byte for byte into `.specsync/archive/closed/<date>-<id>/` with a
`close.json` record of the actor, the reason, the state it was in, its workflow version, the path
it came from, the change that replaces it, any relayed decision, and `reviewed: false`, without
adding a change state and without reviewing, accepting or writing into any spec (#1070, #844).
A closed change SHALL count toward no check, coverage, archive preflight, successor walk or
sequence accounting, and its ID SHALL stay reserved. Several changes SHALL close together or
none. A change whose materialized spec text has not merged into the remote default branch SHALL
be refused, naming what to restore or remove.

#### Acceptance Criteria

- A draft and an approved change closed in one command move under `.specsync/archive/closed/`
  with their `state.json` bytes kept and a `close.json` recording who, why and the state.
- `change status`, `show` and `ship-status` report the close; `change check` refuses the change;
  `change list` omits it; `change new` refuses its ID.
- One unknown ID in the batch closes none.
- `--replaced-by` must name an existing change and is recorded.
- A checked change whose materialized spec text differs from `origin/main` is refused, naming
  the checkout for files `origin/main` holds and removal for files the change created.
