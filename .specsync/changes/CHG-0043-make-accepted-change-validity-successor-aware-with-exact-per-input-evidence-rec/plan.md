---
change: CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec
artifact: plan
---

# Plan

1. Add backward-compatible manifest and semantic-succession types with domain-separated closing-digest binding.
2. Refactor aggregate calculation to derive from reusable exact per-input entries, including post-delta overrides and historical sequence-ledger behavior.
3. Resolve signed owners from the immutable post-delta canonical snapshot, reject unmapped source inputs, and mark known delivery metadata exact-only.
4. Add supported digest-bound supersedes edges and generate exact predecessor/path/module/old/new succession tuples only from approved obligations, semantic deltas, and trusted tree transitions.
5. Index active and archived records and implement a memoized recursive terminal-validity graph with cycle rejection.
6. Replace both permissive successor helpers with exact per-entry same-successor tuple validation.
7. Route check, status, reopen, and archive through the shared validity context.
8. Add accepted-transition-anchored and content-deduplicated Git-tree reconstruction for stale legacy aggregates and legacy semantic transitions while preserving exact legacy closing bytes.
9. Update change and command canonical contracts, including removal of implementing/verifying masking from `REQ-change-024`.
10. Add the full unit and CLI regression matrix, then run focused tests, the native Fledge verification lane, strict SpecSync validation, integration status, Trust doctor, and Trust verify.
