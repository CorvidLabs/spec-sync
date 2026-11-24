# Lesson bundle: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Rebase a stale MODIFIED requirement delta automatically when main's edit does not overlap it, keeping the approval
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change.rs, src/change/, src/change_tests.rs, src/change_tests/, tests/integration.rs, tests/integration/, specs/change/, site/src/content/docs/, CHANGELOG.md
- **Acceptance**: When main edits a requirement a MODIFIED delta was approved against and the edits do not overlap word by word, change check writes the word merge into the delta as a proposal and refuses until a person approves it with change approve, after which change check applies it; when the edits overlap, main deleted the requirement or edited its heading, the working tree holds other text, the requirement is too long, or Git cannot read main's text, it refuses as today naming the cause and the way on, and writes nothing; nothing is proposed in CI

## Evidence

- Verification commit: `2cda672f91ebf35e56e4a2cdf56860759523372d`
- Base commit: `99f61b2b5667d60c3b0705902486f910c8e067c5`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

spec-sync #1085, from the Chough 0.6.2 work. A MODIFIED delta carries a requirement's whole text. When main changes the same requirement after the delta was approved (CorvidLabs/chough #336, #350 and #344), SpecSync refuses the stale delta (#843, REQ-change-122), so the agent rebases it by hand and the human approves it again, even when the two edits do not overlap.

Leif approved the issue on 2026-11-18 ("Leif's decision (2026-11-18): approved.", written on the issue), and hc t4c relayed it to this agent (claude-code, hc t4t) as his scope approval. Limits: no release, tag or publish, and no merge without his relayed yes.

What a session picking this up needs to know:

- The base text is not stored. The approval records only a digest of the block (#843). #975 already recovers the approval-time text from Git by that digest (`recover_approved_base_body`) and compares line by line (`three_way_keeps_body`), to let a re-approval keep a body that already carries main's edits. This change reuses that recovery and the same LCS alignment, word by word, and produces the merge rather than a yes or no.
- The approval binds each module's delta by digest (`approved_delta_digests`) and each replacing item's base (`approved_delta_bases`). Under workflow v2 the definition digest does not hash delta bodies, so a rebase can keep the approval through a record that moves those two claims. Under workflow v1 it does hash them, so v1 is left out.
- This branch is stacked on #1084 (PR #1086), so its own check writes history fragments.

## Option 2: propose, don't approve (2026-11-19)

After five review rounds, Leif chose a simpler rule ("1089 option 2", relayed by hc t4c at about 14:43Z, and "approve 1089" on the wording at about 14:59Z, frozen in ss-lead/gates/1089-option2-definition.md). `change check` writes the word merge into the delta as a proposal and refuses; only a person's `change approve` makes it the approved body. The rebase records, the recomputing readers and the provenance checks are dropped; the merge and its safety checks stay. The change id still says "keeping the approval": ids never change, and that wording no longer describes it. The send-back that reopened the definition was run by the builder on Leif's relayed word ("yes to both", 15:02Z).

## From the change's design.md

# Design

- **Where.** `change check` (`materialize_change_deltas_inner`), after the read-only gates (dependencies, conflicts, open tasks) and before the base check, outside CI.
- **Main's block** is read from Git at the merge-base of `HEAD` and the default branch, a commit on its first-parent line. Main has edited the item when that block, heading included, differs from the approved base and from the change's own text and the line held the base earlier, or when main deleted the block after holding it. A block equal to the change's own text (#654) is not main's edit.
- **Merge.** Word tokens; anchors are words unique in all three texts, chained by the longest rising sequence and found again in each gap. Refused as an overlap: a stretch both sides changed differently with no anchor to split it; a word both sides removed from one stretch the same way that one side alone adds elsewhere (moves); a merged word count outside the range from the base's count plus both sides' deletions to the base's count plus both sides' additions. A side over 4,000 words is too long to merge.
- **Proposal.** When every stale item merges cleanly and each working-tree block is main's block, the delta's body or their merge, the merged bodies are written into the delta (only the delta), all or nothing, and the check refuses, naming each requirement and `specsync change approve <id>`. The delta then differs from its approval, so the delta-body binding refuses on any later check until the person approves; `change approve` records the merged body with the canonical block as its base, and the next check applies it.
- **Refusals** name the cause and the way on: overlap, deletion, heading, other text or too long (rewrite onto main's text by hand, then approve); an unreadable merge-base, tree, text or history with a default branch (repair or fetch, then check again). With no default branch, and in CI, nothing is proposed and an applied item reads as unedited.
- **Dropped from the earlier rounds:** the `delta_rebases` records and every reader that folded them in, the provenance checks on a record's commit, status's rebase lines, the 256 KiB and 64-record bounds, and writes to the canonical requirement during a rebase.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-rebase-proposal-001 | `src/change_tests/rebase_stale_delta.rs`: a non-overlapping main edit is written into the delta as a proposal and refused; after `change approve` the check applies it and the requirement carries both edits; the canonical file and `approvals.json` are untouched by the proposal; each working-tree text (main's, the change's, the merge) gets the same proposal; overlaps, moves, a lost repeated word, main's deletion, main's heading edit, branch-side text, an older main text, a side branch's text and an unreadable merge-base are refused naming the cause; the #654 no-op; no proposal in CI or without a default branch; the 4,000-word limit; `tests/integration/rebase_stale_delta.rs` through the binary |
| REQ-change-122 | the existing `stale_modified_delta` tests, still refusing overlapping edits |

## How it runs

Unit and integration tests locally (red, then green), the full verify lane, and the real-repo run on the open changes of chough, starling and the seven triples' repositories.

## Where these lessons go

- `specs/change/change.3md`
