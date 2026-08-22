---
change: name-the-lifecycle-you-are-on-and-record-the-archive-that-proves-it
artifact: design
---

# Design

## Predicate

`accepted_change_is_recorded_in_ref` splits into a shared body parameterised by (path, expected
state), then tries two locations:

1. the active workspace path, expecting `Accepted` — unchanged behaviour
2. for archived records only, the archive workspace path resolved through `find_change_dir`,
   expecting `Archived`

Each location is asked for the state it can actually hold. A record cannot be `Accepted` at its
archive path, so reusing the old expectation there would have silently matched nothing.

## Announcement

`print_change_text_identity` gains a notice printed only when `workflow_version < 2`. v2 stays
silent, so the healthy path gains no output. `init` gets the same treatment, guarded on the
baseline file being absent and the policy version being below 2.

The notice names the lifecycle and what it implies for verbs (`accept`/`archive` rather than
`finalize`), then points at `change adopt`. Naming state first is deliberate: a reader who believes
they are on v2 needs the assumption contradicted, and a verb pointer does not contradict anything.
