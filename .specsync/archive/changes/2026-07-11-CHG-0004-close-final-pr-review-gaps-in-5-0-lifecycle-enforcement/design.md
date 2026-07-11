---
change: CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement
artifact: design
---

# Design

Keep lifecycle enforcement centralized in `change.rs`:

- coverage collection returns a `BTreeSet` union of base, index, worktree, and untracked paths for local runs;
- covering states are an explicit predicate;
- accepted-state validation loads verification and approval ledgers and recomputes the closing digest;
- delta validation compares on-disk module files with the declared affected-spec set and rejects unknown operation headings;
- acceptance reruns delta validation before preparing writes;
- dependency conflict exemption uses graph reachability;
- version bumping recognizes either an integer or exactly three numeric semantic components.

The check command keeps its established top-level JSON keys and adds `sdd`. Initialization passes detected source directories into policy construction. `.specsync/` implementation files remain ignored as a group, while a protected-path check makes committed policy/config files meaningful before ignores are evaluated. Policy transitions are evaluated with the trusted base policy and require delivery-state coverage of the policy file.
