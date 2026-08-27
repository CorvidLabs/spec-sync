# Lesson bundle — audit-every-module-context-md-against-the-source-and-correct-every-false-or-stale-claim

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Audit every module context.md against the source and correct every false or stale claim
- **Kind**: Documentation
- **Specs**: agents, change, changelog, cmd_agents, cmd_check, cmd_comment, cmd_coverage, cmd_deps, cmd_diff, cmd_hooks, cmd_init, cmd_init_registry, cmd_new, cmd_report, cmd_scaffold, cmd_score, cmd_wizard, commands, comment, deps, exports, git_utils, github, hooks, ignore, importer, manifest, mcp, merge, output, parser, registry, schema, validator
- **Paths**: specs/agents/context.md, specs/change/context.md, specs/changelog/context.md, specs/cmd_agents/context.md, specs/cmd_check/context.md, specs/cmd_comment/context.md, specs/cmd_coverage/context.md, specs/cmd_deps/context.md, specs/cmd_diff/context.md, specs/cmd_hooks/context.md, specs/cmd_init/context.md, specs/cmd_init_registry/context.md, specs/cmd_new/context.md, specs/cmd_report/context.md, specs/cmd_scaffold/context.md, specs/cmd_score/context.md, specs/cmd_wizard/context.md, specs/commands/context.md, specs/comment/context.md, specs/deps/context.md, specs/exports/context.md, specs/git_utils/context.md, specs/github/context.md, specs/hooks/context.md, specs/ignore/context.md, specs/importer/context.md, specs/manifest/context.md, specs/mcp/context.md, specs/merge/context.md, specs/output/context.md, specs/parser/context.md, specs/registry/context.md, specs/schema/context.md, specs/validator/context.md
- **Acceptance**: No specs/<module>/context.md asserts a symbol, file path, or test name that does not exist in the tree: check_project_quiet, auto_regen_stale_specs, remove_section, src/exports.rs, build_schema in validator.rs, and the deleted CI lifecycle workflows are all gone or restated as history.
- **Acceptance**: Every count a context.md states is the number a stated command produces today: tracked .md files, archived approval ledgers, unit and integration test totals, exports source files, and the cmd_coverage / cmd_diff / cmd_report integration-test counts.
- **Acceptance**: No context.md claims a behaviour the code contradicts: the change-sequence ledger is described as written by floor_sequence_ledger_to_committed rather than as read-only, deps records rather than silently swallows unreadable input, registry parses TOML with the toml crate, output is not claimed to do no file I/O, and coverage reports a zero denominator as null rather than 100%.
- **Acceptance**: Claims that were true only before a later change are marked as history rather than left in the present tense: parser.rs handling CRLF, CHG-0063 and CHG-0066 being under verification, and the CI lifecycle reimplementation deleted by #499.
- **Acceptance**: Judgement, rationale, and historical narrative are left untouched; only factual assertions about the current codebase are edited.

## Evidence

- Verification commit: `dccd82105956d62df76bb4fec9fb777c4b31f15b`
- Base commit: `dccd82105956d62df76bb4fec9fb777c4b31f15b`
- Verified by: `cargo test agents::`, `cargo test change::`, `cargo test commands::check::tests::`, `cargo test commands::tests::`, `cargo test exports::typescript::`, `cargo test exports::erlang::`, `cargo test exports::ast::erlang::`, `bash .github/scripts/test-classify-ci-paths.sh`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`, `cargo test hooks::tests::`, `cargo test ignore::tests::`, `cargo test schema::tests::`, `cargo test validator::tests::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`

## From the change's context.md

# Context

Issue #714 found two false statements in `specs/change/context.md`, both folded in by the lessons
loop working exactly as designed, and closed with one sentence that scoped this change:
**"Nobody has audited the rest."**

A wrong lesson is not an ordinary wrong comment. `change new` puts a module's `context.md` in front
of an author BEFORE they scope anything, which is the whole point of #697. Knowledge arriving with
the authority of recorded experience, at the moment decisions are made, is load-bearing — and it is
harder to dislodge than a wrong code comment, because the reader has no diff to review it against.
The loop has no notion of truth, only of provenance.

Sixty-two `context.md` files, 2,549 lines, most of it older than the two corrections. This change
reads every checkable claim in all of them against the source and fixes the ones that are wrong.

## What made a claim checkable

A file or symbol exists; a named test exists; a count; a convention ("all N call sites do X"); a
pointer to a module or line; a behaviour settled by reading one function. Everything else —
judgement, rationale, the narrative of a past decision — is legitimate prose and was left alone.
Most of these files are mostly that, which is why 34 of 62 needed an edit and 28 did not.

## Constraints this worked under

- **Verify, never assess.** Every verdict here came from running a command or reading the source.
  Plausibility is what produced the original defect.
- **Recount every number.** The two known-false lessons died because a count was carried over with
  a different denominator than the reader would assume. No number was inherited, including the
  corrected `21 of 39` from #714.
- **Do not over-correct.** A claim that is imprecise but sound in context stays. Turning a
  true-but-loose statement into a differently wrong one is the same failure with the sign flipped.
  Several claims were examined and deliberately left: see `research.md`.
- **Fix the lesson, not the loop.** #714 offers four designs (cite evidence not conclusions; date
  and attribute; re-derive on read; correct loudly). All four are out of scope. This change
  corrects wrong statements and implements none of them.

## Prior attempts and what is already ruled out

- #696 corrected the two `specs/change/context.md` lessons #714 names. Both were folded here before
  the correction landed; the loop propagated the pre-correction version faithfully.
- #732 swept the `CHG-NNNN` allocation vocabulary out of `specs/change/change.spec.md` and two
  `context.md` paragraphs. It caught the allocation wording — and left `Nothing writes it any more`
  standing, which is false for a different reason (see `research.md`).
- #733 unified the three frontmatter readers behind one delimiter rule. Every frontmatter claim
  here was re-checked against the merged state, not against the state #714 described.

## Scoping note worth carrying

This change declares its 34 owning specs AND `--no-spec-change`. Declaring `--spec` alone makes
`validate_delta_files` require exactly one semantic delta per affected spec, and a delta rewrites
the canonical `<module>.spec.md` — which this change does not touch. `--no-spec-change` short-
circuits that check while `--spec` keeps ownership explicit, which is the combination
`docs/ADOPTING.md` documents for "the modules own these paths, but no canonical spec text moves".

## From the change's design.md

# Design

## The editing rule

A correction replaces a false statement with the true one and stops. It does not restructure the
paragraph, does not add evidence-citation scaffolding, and does not rewrite judgement that happens
to sit next to a wrong fact.

Where a claim was true when written and a later change falsified it, the correction preserves the
history rather than deleting it — `check_project_quiet` is named as removed by #543, the CI
lifecycle workflows as deleted by #499, `auto_regen_stale_specs` as removed with embedded
inference in #335. A reader who followed the old lesson needs to know it was true once and what
replaced it; silently swapping the text leaves them unable to reconcile what they remember.

## Three shapes of defect, and how each was handled

**Dead pointer.** A named symbol, path, or test that does not exist. Replaced with the real name.
This is the cheapest class to find and the least dangerous — a broken pointer is visible.

**Decayed count.** A number that was right and drifted. Replaced with a measured value and the
counting basis, so the next reader can tell what denominator it uses. Where the count is anchored
to an issue and phrased in the past tense (`0 of 107 archived reviews (#694)`), it was left: it is
a record of a measurement, not a claim about today.

**Reversed behaviour.** The dangerous class, because every symbol in the sentence still exists.
`Nothing writes it any more` names a real file, uses correct vocabulary, and is false;
`floor_sequence_ledger_to_committed` writes it from inside every lifecycle commit. Finding these
required reading the function, not grepping the name.

## Counts that decay, and why they were not simply deleted

Several corrected numbers will drift again — 2263 tracked `.md` files, 188 of 202 ledgers, 2407
unit tests. Removing them would stop the decay and also remove the only thing that makes the
surrounding claim checkable. A wrong number is worse than a decaying one: it is confidently
precise. Each corrected count now carries its counting basis in the sentence, so the next reader
can re-run it instead of trusting it, and `research.md` records the command.

## Where the loop's own residual sits

`21 of 39 call sites normalize` is the corrected figure from #714, and no command recorded
anywhere reproduces it exactly (see `research.md`). It was not changed, because three defensible
methods give 21, 22 and 24 and none shows it wrong. That is the honest state of it — and it is the
best available argument for #714's option 1: a lesson that cites its command is re-derivable, and
a lesson that cites only its conclusion is not, even when the conclusion is right.

## Scoping

`--spec` for all 34 owning modules AND `--no-spec-change`, which `docs/ADOPTING.md` documents as
the combination for "these modules own the paths, but no canonical spec text moves". `--spec`
alone would make `validate_delta_files` demand one semantic delta per affected spec, and a delta
rewrites the canonical `<module>.spec.md` this change does not touch. Dropping `--spec` (the
fold-back recipe in `ADOPTING.md`) would leave 34 edited modules with no declared owner; this is a
correction audit, not a fold-back of an archived bundle, so that recipe's rationale would not be
true of it.

## From the change's testing.md

# Testing

**No test is added, and none is possible.** This change edits prose in 34 `context.md` companion
files and touches no source, no behaviour, and no canonical spec text. There is nothing on either
side of the edit for an assertion to discriminate between.

The discrimination protocol requires that a new assertion be shown to FAIL against a binary built
from a separate checkout of unfixed `main`. Any test written for this change would either:

- assert on the prose itself, which passes on unfixed `main` for one file and fails for another
  purely because the string differs — a change-detector, not a discriminator; or
- assert on the code the prose describes, which passes identically on both sides, because the code
  is what was already correct. The prose was the thing that was wrong.

Saying that plainly is the honest outcome. Adding a test that cannot fail for the right reason
would misrepresent this change as behaviour-verified.

## What was verified instead

The evidence for this change is the reproduced measurement, not a test. Every count and every
symbol claim is recorded in `research.md` with the command that produced it, so a reviewer can
re-run each one against this tree and get the number now written in the file. That is the whole
verification surface, and it is deliberately re-runnable rather than pinned.

## Gates that must still pass

- `cargo fmt --check` — clean (no source changed).
- `cargo clippy -- -D warnings` — clean (no source changed).
- `cargo test` — 2,407 unit and 407 integration tests, unaffected by a prose-only change; run to
  prove exactly that.
- `specsync change check` — targeted verification for this change.
- `specsync change audit --strict` — exit 0.

## One characterization worth recording

`tests/integration/comment.rs::comment_suppresses_configured_command_output_but_check_streams_it`
still passes, but no longer for the reason its name gives: since #543, `comment` runs no configured
verification command at all, so the absence of that output is trivial rather than suppressed. The
test is not touched here — it is outside this change's scope — but `specs/cmd_comment/context.md`
now says what it actually proves, so the next reader does not mistake it for evidence of a quiet
execution path that no longer exists.

## Where these lessons go

- `specs/agents/context.md`
- `specs/change/context.md`
- `specs/changelog/context.md`
- `specs/cmd_agents/context.md`
- `specs/cmd_check/context.md`
- `specs/cmd_comment/context.md`
- `specs/cmd_coverage/context.md`
- `specs/cmd_deps/context.md`
- `specs/cmd_diff/context.md`
- `specs/cmd_hooks/context.md`
- `specs/cmd_init/context.md`
- `specs/cmd_init_registry/context.md`
- `specs/cmd_new/context.md`
- `specs/cmd_report/context.md`
- `specs/cmd_scaffold/context.md`
- `specs/cmd_score/context.md`
- `specs/cmd_wizard/context.md`
- `specs/commands/context.md`
- `specs/comment/context.md`
- `specs/deps/context.md`
- `specs/exports/context.md`
- `specs/git_utils/context.md`
- `specs/github/context.md`
- `specs/hooks/context.md`
- `specs/ignore/context.md`
- `specs/importer/context.md`
- `specs/manifest/context.md`
- `specs/mcp/context.md`
- `specs/merge/context.md`
- `specs/output/context.md`
- `specs/parser/context.md`
- `specs/registry/context.md`
- `specs/schema/context.md`
- `specs/validator/context.md`
