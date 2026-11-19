---
change: rebase-a-stale-modified-requirement-delta-automatically-when-main-s-edit-does-not-overlap-it-keeping-the-approval
artifact: context
---

# Context

spec-sync #1085, from the Chough 0.6.2 work. A MODIFIED delta carries a requirement's whole text. When main changes the same requirement after the delta was approved (CorvidLabs/chough #336, #350 and #344), SpecSync refuses the stale delta (#843, REQ-change-122), so the agent rebases it by hand and the human approves it again, even when the two edits do not overlap.

Leif approved the issue on 2026-11-18 ("Leif's decision (2026-11-18): approved.", written on the issue), and hc t4c relayed it to this agent (claude-code, hc t4t) as his scope approval. Limits: no release, tag or publish, and no merge without his relayed yes.

What a session picking this up needs to know:

- The base text is not stored. The approval records only a digest of the block (#843). #975 already recovers the approval-time text from Git by that digest (`recover_approved_base_body`) and compares line by line (`three_way_keeps_body`), to let a re-approval keep a body that already carries main's edits. This change reuses that recovery and the same LCS alignment, word by word, and produces the merge rather than a yes or no.
- The approval binds each module's delta by digest (`approved_delta_digests`) and each replacing item's base (`approved_delta_bases`). Under workflow v2 the definition digest does not hash delta bodies, so a rebase can keep the approval through a record that moves those two claims. Under workflow v1 it does hash them, so v1 is left out.
- This branch is stacked on #1084 (PR #1086), so its own check writes history fragments.
