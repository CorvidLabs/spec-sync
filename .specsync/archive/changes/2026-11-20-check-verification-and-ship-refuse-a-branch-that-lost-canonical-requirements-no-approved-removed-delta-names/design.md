---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: design
---

# Design

- **Reference commit.** The merge-base of `HEAD` and the default branch (`origin/HEAD`, else `origin/main`, else `origin/master`); with no default branch, the change's `base_commit` when it is in `HEAD`'s history; with neither, skip with one note. The merge-base moves forward when the branch merges main, so requirements another change removed on main are not this branch's loss.
- **Requirement ids.** Every `### REQ-` heading outside a fenced block (the change module's own fence tracking plus `crate::validator::requirement_heading_id`) in each `requirements.md` and legacy `<module>.req.md` under the specs directory and beside each registered spec, at the reference commit (read from Git) and in the working tree. Lost = an id at the reference that is in no such file now. A move under the same id is not lost.
- **Accounting.** The approved `## REMOVED` requirement items of every change on the branch: this change, every other active change whose definition approval is valid, and every change archived on the branch after the reference commit (its archived package's deltas).
- **Refusal.** One message naming each unaccounted id and the file that held it at the reference, the way out (`git show <commit>:<file>` to restore; a `## REMOVED` item and a fresh approval to remove on purpose), and, when the lost block's body is in the working tree under another id, the permanence hint. A refused check records a refused attempt (#847).
- **Where.** In a new `src/change/vanished_requirements.rs`, called beside `ensure_approved_delta_bases_current` at `change check` (materialization), acceptance (which finalize and ship reach), workflow-v1 `change verify`, and the audit's per-change chain; `change ship-status` lists it as a blocker (REQ-cmd-change-042).
- **Rejected.** Comparing with `base_commit` (blames the branch for merged-in main removals); with the approval-time canonical text (misses a truncation committed before approval, and is not stored); touching `src/validator/requirement_ids.rs` (its fence helper is private, and declaring it would need the `validator` spec in scope).
