# Lesson bundle: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Require a fresh review after a send-back even when the scope comes back unchanged
- **Kind**: BugFix
- **Specs**: change
- **Paths**: CHANGELOG.md, hi/change.md, src/change/, src/change_tests/, tests/integration/change.rs
- **Acceptance**: A change reviewed, sent back and re-approved with its scope unchanged, then checked without --commit, is not ready to ship: change status names change review, change ship-status reports the review stale, and change ship and change finalize refuse until a fresh review is recorded; the fresh review then finalizes the change with both review attempts in its history, the first unchanged.
The integration test built from the #960 scenario (the Codex P1 on #944) fails on main 8a251d7a and passes with the fix.
A send-back records how many review attempts preceded it, beside cancelled_approvals, and a review counts only when it was recorded after the latest send-back; a send-back recorded before that field existed falls back to a strict timestamp comparison, so no existing archive changes its verdict.
Review history validity (#914), the CHANGE-12.b currency rule and every other review currency check are unchanged; no gate, bound or timeout is relaxed.

## Evidence

- Verification commit: `e271444fda7bb443129857ddc1fd64279a000ae3`
- Base commit: `8a251d7a553f5c98dc1f2f505262f4b0d055e3f7`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #960, from the Codex review of #944 (#914). A scoped review binds a definition approval by digest. When a reviewed change is sent back and re-approved with an unchanged scope, the new approval has the same digest. If `change check` then runs without `--commit`, `HEAD`, the execution digest and the project-input digest stay the same too. The earlier review then binds the post-send-back approval by digest, `scoped_review_currency` returns `Current`, and `change ship`/`finalize` pass without a fresh review.

Reproduced on main 8a251d7a through the built binary under workflow v2 (`a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`): after `approve → check --commit → review → send-back → approve → check`, status reports `scoped_review_current: true` and the next action `finalize`.

Leif's intent, captured verbatim as CHANGE-12.e: "After I send a change back, it needs a fresh review before it can ship, even when its scope comes back unchanged." This was main's behaviour before #944 too; #914's approved scope named it out of scope, and Leif chose to fix it in its own change.

Out of scope: review-history validity (#914), the send-back command's other effects, and what a review must match beyond being recorded after the latest send-back.

## From the change's design.md

# Design

A send-back already records `cancelled_approvals`, the number of approvals before it, which is how the definition gate ignores an approval the send-back cancelled. The review side gets the same treatment.

- **Record:** `SendBackRecord` gains `cancelled_review_attempts: Option<usize>`, the number of entries in `review-attempts.json` when the send-back is written. It is written on every new send-back and is absent on send-backs recorded before this change. Old records keep their committed bytes; the committed-send-back immutability check is unchanged.
- **Currency:** `scoped_review_currency` refuses a review that was not recorded after the latest send-back, before the contract digest check, with a stale reason naming the send-back and a fresh `change review`.
  - When the latest send-back carries the count *n*, the review counts only if it is one of the attempts after the first *n*.
  - When it does not (a send-back recorded by an older SpecSync), it counts only if its timestamp is strictly later than the send-back's. Every archive on main with a send-back has its review recorded after it, so no existing verdict changes.
- **Unchanged:** review-history validity (#914) still binds any recorded approval. The CHANGE-12.b check, the contract, execution, project-input and commit checks are unchanged. No gate, bound or timeout is relaxed.

Rejected:
- Timestamps alone: a send-back and a fresh review can land in the same second in scripted flows and tests, which makes the order ambiguous.
- Binding a review to an approval ledger index: that needs a new field on every review record, and an old review would still need a fallback. The send-back is the event that cancels, so it is where the count belongs, as with `cancelled_approvals`.
- Dropping or rewriting earlier attempts: CHANGE-12.c and append-only history forbid it.

## From the change's testing.md

# Testing

Failing test first, through the built binary, under workflow v2.

1. Integration (`tests/integration/change.rs`): `a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`. Approve, `check --commit`, review, send-back, approve unchanged, `check` without `--commit`.
   - Status names `change review`.
   - ship-status reports the review stale.
   - `change ship` refuses and leaves the change active.
   - A fresh review then finalizes with both attempts, the first unchanged.
   - Fails on main 8a251d7a (status says `scoped_review_current: true` and `finalize`).
2. Unit (`send_back_for_rescope.rs`):
   - the send-back records the attempt count;
   - after an identical re-approval the earlier review is `Stale` for the send-back reason, and the fresh one is `Current`;
   - a legacy send-back without the count uses the strict timestamp rule.
3. The existing send-back, scoped-review, committed-history and #914 tests are unchanged and green.
4. `specsync change check <id> --commit`, `specsync change audit --strict`, `fledge lanes run verify` on pinned 1.89, pre-push.

## Results

- The integration test failed on main 8a251d7a (`scoped_review_current: true`, next action `finalize`) and passes with the fix.
- The two new unit tests pass, and the 15 existing send-back and scoped-review unit tests are unchanged and green.
- The send-back and review integration tests (5), #914's included, are green.

## Where these lessons go

- `specs/change/change.3md`
