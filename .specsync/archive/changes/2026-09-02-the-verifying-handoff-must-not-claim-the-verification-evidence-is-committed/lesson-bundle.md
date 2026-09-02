# Lesson bundle — the-verifying-handoff-must-not-claim-the-verification-evidence-is-committed

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The Verifying handoff must not claim the verification evidence is committed
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs
- **Acceptance**: After a passing change check without --commit, the record is Verifying with verification.json untracked and current. The Handoff line then reads safe with a reason that says the implementation is committed and its verification is current; it no longer says the verification is committed, because HandoffSignals carries no such signal and classify_handoff never checked it. A unit test pins that no Verifying reason claims the verification is committed. The verdict, resume command, and JSON shape are unchanged.

## Evidence

- Verification commit: `bd1c038a7de8222ba7bc6b9db4bda26ce7bf12e7`
- Base commit: `d929a60d70903c0241d3a6411961ea168004d561`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Found while running the coverage record on this branch through the lifecycle. After
`specsync change check <id>` (without `--commit`) the record was Verifying with
`verification.json` untracked, and the new `Handoff:` line read:

    Handoff: safe — verification is committed and current; a fresh session resumes at the independent review

The verdict is right: `ship-status` reports the product tip as done and the review stage as
current from uncommitted evidence, and a fresh session reads `verification.json` from disk, so
nothing this session knows is lost by clearing. But `classify_handoff` reaches that arm on
`verification_current` (content currency, `recorded_verification_is_current`) and
`scoped_edits_uncommitted != Some(true)` (the implementation paths are clean). `HandoffSignals`
has no "verification committed" signal, so the reason asserts something the classifier never
checked, and does so in the most common state — every `change check` run without `--commit`.

The fix is wording only: say what was checked — the implementation is committed and its
verification is current. The Approved/Implementing arm already words it that way.

Ruled out: making the arm `conditional` with "run `change check --commit`". The lifecycle does
not require the evidence to be committed before review, and telling agents to commit between
review and finalize is exactly what `ship-status` warns against.

## From the change's testing.md

# Testing

`handoff_verifying_follows_evidence_currency` already walks Verifying through stale evidence
(conditional), current evidence awaiting review (safe), and current review (safe). It gains
one assertion per safe arm: the reason must not contain "verification is committed". That
fails on the current binary — the awaiting-review arm carries exactly that phrase — and passes
after the reword.

Discriminator: `handoff_verifying_follows_evidence_currency` (fails before, passes after).
Control: `handoff_follows_the_lifecycle_and_ignores_uncommitted_lifecycle_evidence`, which
drives a real repository through the same state with `verification.json` uncommitted and
still expects `safe` — the verdict is untouched.

Run: `cargo test --release --bin specsync handoff` (12 tests) and the integration test
`status_prints_a_handoff_line_and_json_carries_it`.

## Where these lessons go

- `specs/change/context.md`
