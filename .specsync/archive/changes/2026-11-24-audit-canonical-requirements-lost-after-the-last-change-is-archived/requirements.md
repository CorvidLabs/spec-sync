---
change: audit-canonical-requirements-lost-after-the-last-change-is-archived
artifact: requirements
---

# Requirements

A branch can lose a canonical requirement after its last change is archived. CHANGE-17.d covers both `change audit` and `check --strict`. Each command should report every unexplained loss even with no active workspace.

Reuse REQ-change-169 for canonical-file selection, permanent IDs, bounded reads and approved removals. An authorized removal archived on this branch after the comparison base accounts for its own ID. An older archive grants no authority for a later deletion. Moving a requirement between canonical files under the same ID preserves it.

An ordinary `check` continues checking drift. With `--strict`, the shared historical comparison applies to the managed branch even when CI invokes only strict check. It reports requirement loss without running lifecycle audit or reporting unrelated workspace and archive findings. Projects that have never enabled SDD retain their current strict drift checks and need no removal deltas.
