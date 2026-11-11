---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: research
---

# Research

- #1043 repro: a main build records relayed approvals and a send-back; one `change send-back` by the
  released 6.2.0 strips every `relayed_by` and `cancelled_review_attempts` (exit 0); the main build
  then reports "a committed send-back was removed or edited" and `approval_valid: false`.
- #434 part 1 (still reproducing on main): any typed rewrite drops unknown members, at the top
  level and inside entries, including `verification-attempts.json` history when appending.
- `serde_json::Value` is not usable for the merge: `preserve_order` is enabled only through a
  build-dependency (tree-sitter), so the shipped binary sorts object keys, and numbers become floats.
- No persisted lifecycle struct uses serde `alias` or `flatten`, so a member the round trip drops
  is never a renamed known field that would turn into a duplicate key.
- Existing raw-span helpers (`ledger_json_span`) read a validated JSON value without decoding it.
