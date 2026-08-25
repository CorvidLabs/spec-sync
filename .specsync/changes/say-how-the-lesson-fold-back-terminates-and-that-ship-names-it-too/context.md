---
change: say-how-the-lesson-fold-back-terminates-and-that-ship-names-it-too
artifact: context
---

# Context

Issue #703. `finalize` and `ship` instruct the author to fold a change's lessons into
`specs/<module>/context.md` for every affected spec. That fold is itself a change touching
tracked paths, so it needs its own lifecycle record — and if that record declares the same
specs, `lesson_fold_targets` returns the same context paths and the author is told to fold
again. There is no cycle detection, no warning, and no error; the instruction simply repeats.

The loop terminates when a change declares no affected specs: `lesson_fold_targets`
(`src/change.rs:6313`) maps `record.affected_specs` to context paths, so an empty list yields
no targets, and both `lessons_next_action` (`src/commands/change.rs:1236`) and
`ship_next_action` (`src/commands/change.rs:1216`) fall through to their plain merge guidance.
Two archived changes on 2026-08-24 did exactly this and shipped without a fold-back clause:

- `2026-08-24-fold-the-crlf-scaffold-lessons-into-the-change-and-generator-contexts`
- `2026-08-24-fold-the-lessons-loop-bundle-into-the-change-cmd-change-and-generator-contexts`

Both are `kind: documentation`, `affected_specs: []`, `no_spec_change: true`, with only
`specs/*/context.md` in `affected_paths`. Their recorded rationales are the wording this change
generalises.

## What was verified rather than assumed

- `--no-spec-change` requires `--rationale` (`src/change.rs:1479`), so the terminating path is
  the one that forces the author to invent a justification. That is why the wording is supplied
  here instead of left to improvisation.
- A `specs/<module>/context.md` path does not trip the owning-module refusal.
  `path_is_production_source` (`src/change.rs:11595`) requires `exports::is_source_file`, which
  a `.md` companion is not. Declaring `--no-spec-change` with no `--spec` is therefore accepted
  for a companion-only change, and refused for one that also carries production source.
- 6 of 183 archived changes have ever declared a `specs/*/context.md` path (counted over
  `.specsync/archive/changes/*/state.json`). The issue's "4 of 178" was the count before the two
  folds above landed.

## Ruled out

- The behavioural fix the issue floats — `finalize` detecting that a change's only affected
  paths are `context.md` companions and omitting the fold-back clause — is deliberately out of
  scope. It needs a discrimination this change does not build: a change touching a companion
  AND production source must still be told to fold.
- Adding a termination invariant to `specs/cmd_change/cmd_change.spec.md`. Invariant 7 already
  states the tool-side behaviour the termination rests on: "a change owning no affected specs
  receives the same guidance it received before the fold-back existed." What is missing is not a
  tool guarantee but an authoring convention — how to scope the follow-up change — and the tool
  neither enforces nor can verify that. An invariant asserting termination would be claiming a
  contract `change check` cannot hold anyone to. It becomes a real invariant when the structural
  fix above lands and termination stops depending on how the author scopes.

## Correction to the issue

The issue says `ADOPTING.md` "does not mention the fold-back at all". It does — the third bullet
of "Close the learning loop" quotes the `Next:` line and says to do it before merging. What is
absent is the recursion and its termination. The same bullet was also stale: it credited only
`finalize` with naming the step, which #700 changed by making `ship` name it too.

## Verification dead end (first `change check` attempt)

The first `change check` failed on
`commands::gradle_post_discovery_symlink_swap_is_inconclusive_for_every_coverage_gate`
(`tests/integration/commands.rs:1662`) with "check did not reach the root-retained coverage
barrier". Nothing in this change can reach that test — the only delivery path is
`docs/ADOPTING.md`.

It is load-sensitive, not broken. The test spawns a `specsync` subprocess per (phase, command)
pair — ten in total — and waits at most 10 seconds for each to touch a barrier file, polling every
10ms. Under the full suite (405 tests, 592s wall) on a loaded machine, a subprocess can miss that
deadline. Re-run alone it passes in 1.72s.

This is the same failure class as #702 ("a flaky gate teaches everyone to ignore red"): a wall-clock
deadline inside an integration test that is generous when idle and tight when the suite is
saturated. Worth its own issue against the test rather than a retry convention, because a retry is
what teaches people to ignore red.
