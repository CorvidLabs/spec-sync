---
change: CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives
artifact: plan
---

# Plan

1. Reproduce on live work rather than only in a fixture: compare two real changes whose declared
   scopes differ, and show the narrower one received more verification.
2. Isolate the trigger to a single condition by bisecting a throwaway fixture — not test paths,
   not caching, not worktree identity, not a prior failed check.
3. Read `verification_commands_for_change` and confirm the mechanism at the line level.
4. Rule out the innocent explanation: check whether any later lifecycle stage — `check --commit`,
   `review`, `ship-status`, `finalize`, `accept` — re-runs the full set. None does.
5. Establish the blast radius: how many archived records carry narrowed evidence, and whether CI
   independently protects merges on this repository.
6. Decide scope. Fix the monotonicity violation; leave zero-match detection out, because catching
   it requires capturing output and `REQ-change-058` forbids that.
7. Write the property as a superset relation, plus a vacuity control that keeps targeted
   verification alive.
8. Prove discrimination against a separate checkout with `src/change.rs` provably unmodified, and
   check the unfixed build's exit code before trusting its results.
9. Full suite, clippy, fmt. Whole-board check.
10. State the slowdown in the CHANGELOG rather than letting it be discovered.
