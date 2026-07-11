---
change: CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting
artifact: design
---

# Design

Validation occurs at trust boundaries, before persisted values reach path constructors or lifecycle mutations:

- `load_change` and active workspace discovery validate requested IDs, persisted IDs, directory identity, and persisted spec scopes.
- tombstone collection propagates traversal, read, and parse failures instead of constructing incomplete history.
- verifying workspaces require successful, fresh evidence in local and CI checks; CI may additionally rerun configured commands.
- approval append distinguishes a genuinely absent initial ledger from a present but corrupt ledger.
- `specsync comment` merges SDD errors and warnings into the same rendered validation summary and exit decision used by CI.

All checks remain deterministic, shell-free, and cross-platform.
