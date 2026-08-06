---
change: CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes
artifact: research
---

# Research

CHG-0081 declared spec `change` and touched `src/commands/init.rs`, owned by
`cmd_init`. It passed approve and every check, was reviewed, and failed only at
finalize. From there nothing advanced it: `correct-owner` demanded an audited
reopen it never had, `reopen` accepts only Accepted/Archived, and declaring
`cmd_init` would force a semantic delta for a spec the change does not alter.

Sandbox drill 037 reproduces the state in a clean repository in seconds.

Ownership is a property of the proposal, knowable at `change new`. Enforcing it
while building the acceptance manifest put the cheapest possible rejection after
the most expensive possible work.
