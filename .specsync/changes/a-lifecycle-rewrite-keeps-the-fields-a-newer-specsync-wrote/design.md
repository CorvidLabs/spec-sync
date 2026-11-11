---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: design
---

# Design

`json_content_keeping_unknown_members(previous, value)` (src/change/unknown_members.rs) replaces
`json_content` wherever a record or ledger is rewritten in place. It parses three documents into
spans of their JSON text: the file on disk, the typed round trip of that file (what this binary
knows), and the new typed value.

- An object member present on disk and absent from the round trip is unknown. It is appended, with
  its exact text, after the last member of the same object in the new text, unless the new object
  already has that key. A known member the rewrite removed stays removed.
- The root is the same record or ledger before and after.
- An array entry is the same entry only when its known text is byte-equal in the round trip and the
  new value, so an edited entry, or a position that now holds another entry, never gains members.
- The output keeps the layout `serde_json::to_string_pretty` writes; known members keep their order.
- It must parse back to exactly the new value; otherwise the write is refused naming the file and
  the members. A missing or unreadable previous file writes `json_content`'s bytes unchanged.

Fresh records (a new `verification.json`, a new `review.json`, receipts, finalization records)
keep `json_content`. Digests are computed from typed values, so none changes.
