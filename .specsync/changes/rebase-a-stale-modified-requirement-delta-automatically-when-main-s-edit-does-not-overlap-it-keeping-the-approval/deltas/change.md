## ADDED

### REQUIREMENT REQ-change-161

When `change check` finds a `## MODIFIED` requirement item of a workflow-v2 change whose canonical block moved after the item was approved (REQ-change-122), it SHALL try to rebase the item before refusing (#1085). It SHALL recover the block's approval-time text from Git by its recorded digest and merge that text, the current canonical block and the approved body word by word. When the two edits do not overlap, it SHALL rewrite that item's body in the delta to the merge, append a rebase record to the change's `approvals.json` under `delta_rebases`, keep the definition approval, and materialize the merged body. When the edits overlap, or the approval-time text cannot be recovered, it SHALL refuse as REQ-change-122 does and write nothing. A rebase record SHALL hold the approval-time block, main's block and the delta before the rebase, so every reader recomputes the merge and its digests instead of trusting the record; the effective approval then binds the rebased delta and main's block as the item's base, and a record that does not recompute is refused, naming it.

Acceptance Criteria
- Main adds a sentence to a requirement after approval and the change rewords another part of it: `change check` rebases the delta, the canonical requirement carries both edits, the approval stays valid with no `change approve`, and `approvals.json` holds one rebase record.
- Main and the change edit the same words: `change check` refuses as today, naming the requirement and `specsync change approve <id>`, and writes nothing.
- A rebase record whose texts do not recompute to the delta on disk is refused by `change check` and `change status`, naming the record.
- A `## MODIFIED` spec section, a `## REMOVED` item, a workflow-v1 change, a CI run, and an approval-time text Git no longer holds each keep today's refusal.

## MODIFIED

### REQUIREMENT REQ-change-122

A definition approval SHALL record, for every `## MODIFIED` and `## REMOVED` delta item, a digest of the canonical block it was approved against. When such an item is not already applied and its canonical block no longer matches that recorded base, `change check` SHALL refuse before writing any canonical file and `change audit` SHALL report the same refusal, unless REQ-change-161 rebases the item first. The refusal SHALL name the module and block and SHALL name rewriting the delta body onto the current canonical text, `change approve` and `change check` as the remedy. An approval that recorded no bases is unknown, not violated, and keeps materializing as before.

Acceptance Criteria
- A stale MODIFIED item whose block main rewrote after approval, with edits that overlap the change's own, is refused by `change check`, both before and after the first materialization, and the canonical file keeps main's bytes.
- `change audit` reports the same refusal for that change.
- The refusal names the module, the requirement or section, and `specsync change approve <id>`.
- A rebased and re-approved delta materializes, keeping the other change's text it now carries.
- A block main changed that this change does not touch is not refused.
- An approval without recorded bases keeps materializing as before.
