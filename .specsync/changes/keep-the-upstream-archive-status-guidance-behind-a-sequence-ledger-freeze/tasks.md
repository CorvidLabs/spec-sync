---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: tasks
---

# Tasks

- [x] Integration test: upstream archive plus a frozen ledger; JSON next_action, JSON handoff and text Next must all lead with the freeze. Fails on main (handoff).
- [x] Status summary handoff behind the freeze and verification_ignored refusal.
- [x] Text next action and ship-status (blocker and next step): freeze before the upstream archive.
- [x] Deltas, CHANGELOG entry, requirement evidence.
- [x] Carry a `verification_ignored_paths` refusal into `HandoffSignals`, so the handoff names it instead of a lifecycle verdict (#937 part 2, Leif: fix in #934).
