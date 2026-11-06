---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: requirements
---

# Requirements

- REQ-change-001 is narrowed: one recorded scope approval, named for a person, whose digest binds content and whose actor is an unauthenticated claim. It carries CHANGE-6.d.
- REQ-change-relayed-decision-001 covers the record, its validation, byte stability without it, and its exclusion from the review digest. It carries CHANGE-13, 13.a, 13.d and SHIP-2.e.
- REQ-cmd-change-relayed-decision-001 covers the flags, the refusals and the status lines. It carries CHANGE-13.b and 13.c.
- REQ-cli-args-relayed-decision-001 covers the grammar and the `--actor`/`--reviewer` help.

Named requirement IDs are used so parallel rows adding numbered requirements cannot collide with these.
