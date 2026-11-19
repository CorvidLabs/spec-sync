---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: research
---

# Research

Done on 2026-11-19 on the #1084 branch at 99f61b2b.

- `ensure_approved_delta_bases_current` (`src/change/collect_requirement_ids.rs`) refuses any unapplied MODIFIED or REMOVED item whose canonical block digest differs from the approval's recorded base. `materialize_change_deltas_inner` calls it after `ensure_approved_delta_bodies_unchanged` and before anything is written; acceptance and audit call it too.
- `approval_delta_bases` (re-approval) already recovers the approval-time block body from Git (`recover_approved_base_body`, newest 200 commits of the file) and accepts a moved base when a line-level three-way comparison shows the approved body carries main's edits (#975). It never writes a merge.
- The approval binding: `read_approved_delta_binding` compares each module's delta digest to `approved_delta_digests` of `effective_definition_approval`; the bases come from `approved_delta_bases` of the same approval. Acceptance's normalizing approval copies the effective approval's bases forward.
- The existing #843 fixtures (`src/change_tests/stale_modified_delta.rs`) have overlapping edits: both sides change the first sentence's end and both append an acceptance criterion at the end. Under a word-level diff3 they still conflict, so they stay refusals.
- Released 6.3.0 keeps unknown ledger members (#1044), so it would keep `delta_rebases`, but it would read a rebased delta as changed after approval and refuse it. It refuses; it does not rewrite.
