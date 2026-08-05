---
change: CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring
artifact: context
---

# Context

The change module owned its logic but not its command wiring, so any change
touching `src/commands/change.rs` could pass every gate and then fail to
finalize.

The deeper issue is ordering: ownership is a property of the proposal, knowable
at `change new`, but it is enforced at the terminal step after the expensive
verification has already run. The same shape as the archive scope-guard defect
fixed in CHG-0083 — correct enforcement arriving at the least useful moment.
