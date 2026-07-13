---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: research
---

# Research

`accepted_workspace_is_integrated` intentionally requires feature HEAD to be integrated into remote main, which is useful when validating the squash result on main but cannot succeed on a descendant feature branch. Inspecting the deterministic change state through the remote default ref independently proves that the accepted state was actually integrated rather than existing only on an unmerged local branch. Applying that fallback only after current digest and approval checks preserves fail-closed behavior.
