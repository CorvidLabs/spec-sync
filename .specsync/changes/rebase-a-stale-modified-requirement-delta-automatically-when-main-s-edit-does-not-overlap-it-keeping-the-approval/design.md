---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: design
---

# Design

- **Where.** `change check` (`materialize_change_deltas_inner`), after the read-only gates (dependencies, conflicts, open tasks) and before the base check, outside CI.
- **Main's block** is read from Git at the merge-base of `HEAD` and the default branch, a commit on its first-parent line. Main has edited the item when that block, heading included, differs from the approved base and from the change's own text and the line held the base earlier, or when main deleted the block after holding it. A block equal to the change's own text (#654) is not main's edit.
- **Merge.** Word tokens; anchors are words unique in all three texts, chained by the longest rising sequence and found again in each gap. Refused as an overlap: a stretch both sides changed differently with no anchor to split it; a word both sides removed from one stretch the same way that one side alone adds elsewhere (moves); a merged word count outside the range from the base's count plus both sides' deletions to the base's count plus both sides' additions. A side over 4,000 words is too long to merge.
- **Proposal.** When every stale item merges cleanly and each working-tree block is main's block, the delta's body or their merge, the merged bodies are written into the delta (only the delta), all or nothing, and the check refuses, naming each requirement and `specsync change approve <id>`. The delta then differs from its approval, so the delta-body binding refuses on any later check until the person approves; `change approve` records the merged body with the canonical block as its base, and the next check applies it.
- **Refusals** name the cause and the way on: overlap, deletion, heading, other text or too long (rewrite onto main's text by hand, then approve); an unreadable merge-base, tree, text or history with a default branch (repair or fetch, then check again). With no default branch, and in CI, nothing is proposed and an applied item reads as unedited.
- **Dropped from the earlier rounds:** the `delta_rebases` records and every reader that folded them in, the provenance checks on a record's commit, status's rebase lines, the 256 KiB and 64-record bounds, and writes to the canonical requirement during a rebase.
