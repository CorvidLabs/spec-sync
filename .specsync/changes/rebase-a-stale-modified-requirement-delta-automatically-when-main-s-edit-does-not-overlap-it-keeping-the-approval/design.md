---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: design
---

# Design

- **Where.** `change check` (`materialize_change_deltas_inner`), after the delta-body binding check and before the base check, outside CI. For each unapplied `## MODIFIED` `### REQUIREMENT` item of a workflow-v2 change whose canonical block digest differs from its effective base: recover the base block from Git by digest (the #975 helper), merge word by word, and on a clean merge write the rebased delta and the rebase record in one prepared transaction. Then the base check runs on the new claims and passes, and materialization applies the merged body.
- **Merge.** Tokens are a word with its trailing whitespace. Base-to-main and base-to-delta LCS alignments give stable anchors (the #975 loop). Between anchors, a chunk takes the side that changed; both sides changing to different text, including two insertions at one point, is an overlap and refuses. A side over 4,000 tokens refuses.
- **Record** (`approvals.json` `delta_rebases`, append-only, omitted when empty): schema 1, module, item key, the anchor approval's timestamp and digest, the time, the approval-time block, main's block, the module delta before the rebase, and the rebased delta's digest.
- **Effective claims.** `effective_delta_claims(approval, ledger)` starts from the approval's delta digests and bases and folds every record anchored to it, in order. For each record it checks that the pre-rebase delta has the current digest, that the base block has the current base, recomputes the merge and the rebased delta, and checks that digest. Then it moves the module's digest and the item's base to main's block. Any mismatch is an error naming the record. The binding read, the base check, re-approval's previous claims and acceptance's normalizing approval all read the effective claims.
- **Status.** `change status` prints each rebase under the effective approval and lists them in JSON as `summary.delta_rebases`.
- **Rejected:** rewriting the delta and re-recording an approval in the human's name (that would be an approval nobody gave); storing only digests in the record (the record would then be a bare claim that could launder any body); rebasing spec sections and REMOVED items (#1085 names requirements; sections can follow).
