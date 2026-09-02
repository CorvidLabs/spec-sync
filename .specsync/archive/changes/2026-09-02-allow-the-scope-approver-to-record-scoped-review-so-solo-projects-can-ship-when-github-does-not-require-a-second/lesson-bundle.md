# Lesson bundle — allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Allow the scope approver to record scoped review so solo projects can ship when GitHub does not require a second reviewer.
- **Kind**: BugFix
- **Specs**: change, agents, cmd_change
- **Paths**: src/change.rs, src/change_tests.rs, src/agents.rs, src/commands/change.rs, tests/integration/change.rs, docs/ADOPTING.md, AGENTS.md, CHANGELOG.md, specs/change/change.spec.md, specs/agents/agents.spec.md, specs/cmd_change/cmd_change.spec.md
- **Acceptance**: change review accepts the same actor as the definition approver. Solo projects can record review and ship without inventing a second identity. GitHub remains the merge authority for required reviewers. Existing tests that required a distinct reviewer now expect same-actor review to succeed. ADOPTING.md and AGENTS.md no longer tell solo adopters to pick a second identity.

## Evidence

- Verification commit: `37021bac64d46bcbe422509178909f2ffa68635e`
- Base commit: `6a5a7a7d893fb43515c51514989e5b06674656c4`
- Verified by: `cargo test change::`, `cargo test agents::`, `cargo test commands::change::`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

Hit on CorvidLabs/corvid-bot PR 29: GitHub required 0 approving reviews and Trust was green, but `specsync change review --reviewer leif` refused because `leif` had recorded definition approval. SpecSync invented a two-person gate the repository did not have. ADOPTING.md already names this as a solo-adopter bite. GitHub is the merge authority. Scoped review still records who signed off; it must not demand a second identity.

## From the change's design.md

# Design

Remove the case-insensitive `reviewer == scope_approver` refusal in `record_scoped_review_with_verdict` and the matching attempt-history validator. Keep reviewer-claim validation, append-only attempt history, pass/block verdicts, and GitHub as merge authority. Distinct reviewers remain allowed. Same-actor review is allowed. Generated agent skill text, AGENTS.md, and ADOPTING.md drop the "pick a second identity" instruction.

## From the change's testing.md

# Testing

- `scoped_review_requires_an_independent_passing_verdict` must succeed when the reviewer is the definition approver (case-insensitive).
- Invalid reviewer claims still fail (non-ASCII identity).
- Block then pass still appends attempt history.
- `cargo test change::`
- `cargo test`

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-046 | `scoped_review_allows_the_scope_approver_and_records_pass_and_block_verdicts`, `persisted_scoped_review_allows_scope_approver_as_reviewer` |
| REQ-agents-006 | `src/agents.rs` generated skill text; agents install templates |
| REQ-cmd-change-015 | `src/commands/change.rs` next-action copy; `tests/integration/change.rs` |

## Where these lessons go

- `specs/change/context.md`
- `specs/agents/context.md`
- `specs/cmd_change/context.md`
