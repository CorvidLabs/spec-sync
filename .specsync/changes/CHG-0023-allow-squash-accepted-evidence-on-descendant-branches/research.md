---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: research
---

# Research

`accepted_workspace_is_integrated` intentionally requires feature HEAD to be integrated into remote main, which is useful when validating the squash result on main but cannot succeed on a descendant feature branch. `accepted_change_is_recorded_in_current_history` independently proves that an accepted state for the same deterministic change ID exists in branch history. Applying that fallback only after current digest and approval checks preserves fail-closed behavior.
