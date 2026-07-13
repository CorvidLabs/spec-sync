---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: context
---

# Context

SpecSync 5.0.1 rejects stale accepted evidence but exposes no supported recovery transition. The implementation lives in `src/change.rs`; Clap grammar and rendering remain thin adapters. Prior approval and verification evidence must remain portable, inspectable JSON.

Current implementation uses a versioned append-only reopen event and a lifecycle-only `canonical_applied` marker so accepted semantic deltas cannot be applied twice. The marker is excluded from definition digests.
