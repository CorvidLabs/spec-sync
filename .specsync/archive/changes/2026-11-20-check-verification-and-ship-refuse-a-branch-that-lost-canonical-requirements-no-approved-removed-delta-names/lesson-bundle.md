# Lesson bundle: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Check, verification and ship refuse a branch that lost canonical requirements no approved REMOVED delta names
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/vanished_requirements.rs, src/change.rs, src/change/sdd_version.rs, src/change/historical_definition_digest_matches.rs, src/commands/change.rs, src/change_tests.rs, src/change_tests/vanished_requirements.rs, tests/integration.rs, tests/integration/, specs/change/, specs/cmd_change/, site/src/content/docs/deltas.md, CHANGELOG.md
- **Acceptance**: change check, change verify, change accept, change finalize and change ship refuse, writing nothing, a change on a branch that lost a canonical requirement no approved REMOVED delta names, and name every lost id and its file; a removal an approved REMOVED item of a change on the branch accounts for passes, and so does one the branch merged in from the default branch; a requirement moved between files under its id passes, and a renumbered one is refused under its old id with the permanence hint; change audit reports the same refusal and change ship-status lists it as a blocker; the usual flows pass as before

## Evidence

- Verification commit: `b1618923ed5b1181e556c6e06cd14793d1517236`
- Base commit: `3bebe8b03094b2079630801584b56f94a1e460c5`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #1081, moved from CorvidLabs/chough#299. In CorvidLabs/starling, PR #43 added REQ-StarlingCore-33 and merged with a `specs/StarlingCore/requirements.md` holding only that block: REQ-StarlingCore-1 to 28 were gone, no `## REMOVED` delta named them, and `specsync check --strict`, Chough's change check, the native verification and ship, and CI's verify lane all passed. A setup script had truncated the file before appending. It was repaired by hand in starling#46.

A change's approval binds only the blocks its delta names, and the strict check validates exports against code, not canonical requirements against their history, so nothing noticed the loss.

Leif's decision, 2026-11-18 (on the issue): build now, P1. The definition here is the one he approved on 2026-11-19 ("Approve as written", relayed by hc t4c through ss-lead t58), frozen in ss-lead/gates/1081-definition.md.

Constraints for whoever picks this up: compare against the merge-base of HEAD and the default branch, never the working tree's own history; fail closed when a default branch exists and Git cannot read it; account for every change on the branch, not only the one being checked, because refresh branches carry several.

## From the change's design.md

# Design

- **Reference commit.** The merge-base of `HEAD` and the default branch (`origin/HEAD`, else `origin/main`, else `origin/master`); with no default branch, the change's `base_commit` when it is in `HEAD`'s history; with neither, skip with one note. The merge-base moves forward when the branch merges main, so requirements another change removed on main are not this branch's loss.
- **Requirement ids.** Every `### REQ-` heading outside a fenced block (the change module's own fence tracking plus `crate::validator::requirement_heading_id`) in each `requirements.md` and legacy `<module>.req.md` under the specs directory and beside each registered spec, at the reference commit (read from Git) and in the working tree. Lost = an id at the reference that is in no such file now. A move under the same id is not lost.
- **Accounting.** The approved `## REMOVED` requirement items of every change on the branch: this change, every other active change whose definition approval is valid, and every change archived on the branch after the reference commit (its archived package's deltas).
- **Refusal.** One message naming each unaccounted id and the file that held it at the reference, the way out (`git show <commit>:<file>` to restore; a `## REMOVED` item and a fresh approval to remove on purpose), and, when the lost block's body is in the working tree under another id, the permanence hint. A refused check records a refused attempt (#847).
- **Where.** In a new `src/change/vanished_requirements.rs`, called beside `ensure_approved_delta_bases_current` at `change check` (materialization), acceptance (which finalize and ship reach), workflow-v1 `change verify`, and the audit's per-change chain; `change ship-status` lists it as a blocker (REQ-cmd-change-042).
- **Rejected.** Comparing with `base_commit` (blames the branch for merged-in main removals); with the approval-time canonical text (misses a truncation committed before approval, and is not stored); touching `src/validator/requirement_ids.rs` (its fence helper is private, and declaring it would need the `validator` spec in scope).

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-169 | `src/change_tests/vanished_requirements.rs`: the starling truncation refused at check naming every lost id (red on main 3bebe8b0), accounted removals pass (same change, another active change, an archived change, merged from main), a cherry-picked main removal without the merge is refused, a moved block passes, a renumber is refused with the permanence hint, a deleted file and a truncated legacy `<module>.req.md` are refused, the `base_commit` fallback, the skip with a note, an unreadable merge-base refuses, and audit reports it |
| REQ-cmd-change-042 | an integration test: `change ship-status` lists the blocker and exits 1, `change ship --dry-run` and `change finalize` refuse, and restoring the blocks clears them |

## How it runs

`cargo test --bin specsync vanished` and the integration test locally (red, then green), and the full verify lane.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
