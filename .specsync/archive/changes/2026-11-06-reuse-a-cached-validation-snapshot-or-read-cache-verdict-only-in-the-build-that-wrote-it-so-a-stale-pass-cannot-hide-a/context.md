---
change: reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a
artifact: context
---

# Context

- Issue #878 was filed from the #865 review. The duplicate-ID refusal runs in per-spec validation, but the snapshot digest keys only on the version and the format versions. A snapshot from an earlier 6.2.0 build can therefore replay a pass in a default `specsync check` until a file changes.
- Plan row S6 (c). Leif approved CHECK-1.d and approach B (build fingerprint) at 20:20 MST, then "Both in one" at 22:10 MST, so the persistent read cache, which has the same flaw, is fixed here too. Relayed by the orc (t18).
- Human gates: scope approval and implementation review come from Leif through the orc. This agent never records them as Leif.
