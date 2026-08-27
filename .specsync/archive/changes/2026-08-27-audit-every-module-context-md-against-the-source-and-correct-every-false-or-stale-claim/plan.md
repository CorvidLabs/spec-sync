---
change: audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim
artifact: plan
---

# Plan

1. **Enumerate.** All 62 `specs/*/context.md`, 2,549 lines.
2. **Mechanical sweep first.** Extract every backticked token from all 62 files and classify it as
   a path, a Rust symbol, or a requirement/change ID; check each against the tree. This is cheap
   and catches every dead pointer without judgement: it is what surfaced `src/exports.rs`,
   `check_project_quiet`, `auto_regen_stale_specs`, `ChangeClassification::has`, and the
   `git_commits_between` references (the last of which turned out to be correctly written as
   history, not a defect).
3. **Read every file in full.** The sweep cannot see a wrong count, a wrong behaviour, or a claim
   whose symbols all exist but whose sentence is false. Four parallel readers covered the 62 files
   with a fixed rule: extract checkable claims, verify each by running a command, and report the
   command with the verdict.
4. **Re-verify before editing.** Every finding used to justify an edit was re-run here before the
   edit was made. A verdict without a reproduced command was not acted on.
5. **Classify.** TRUE / FALSE / STALE / UNCHECKABLE. UNCHECKABLE is the majority and is not a
   defect.
6. **Fix FALSE and STALE only**, minimally, in the same change.
7. **Freeze scope at `change new`.** Do the whole audit first so the affected file set is known,
   then stash, create the change with the exact 34 specs and 34 paths, and unstash. Scope cannot
   be widened after approval (#542).

## Ordering that mattered

The mechanical sweep ran before any reading. Doing it the other way round biases a reader toward
confirming what the prose says — which is the failure mode under audit. A dead pointer found by
`grep` cannot be talked out of by context.

## What this plan deliberately does not do

- It does not implement any of #714's four designs. Naming the fix for the loop is a separate
  decision and a separate change.
- It does not fix implementations found wrong along the way. Two are reported for their own issues
  (see `docs.md`): a stale source comment in `src/change.rs` and a self-contradicting bullet in
  `AGENTS.md`, both about the same ledger this change corrects in `specs/change/context.md`.
- It does not touch canonical spec text. Where a `context.md` correction revealed a gap in a
  canonical spec (`cmd_score`'s API table omits `min_score`), the context now says so and the
  spec is left for a scoped change.
