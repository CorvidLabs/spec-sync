---
change: make-the-ship-status-product-stage-advertise-only-a-command-its-state-accepts
artifact: tasks
---

# Tasks

- [x] Unit test over every state: the product stage's advertised verb is admitted by that state. Fails on main (draft advertises `check`).
- [x] Product stage action: draft -> `approve`, accepted/archived -> `reopen`, others unchanged.
- [x] Delta, CHANGELOG entry, requirement evidence.
