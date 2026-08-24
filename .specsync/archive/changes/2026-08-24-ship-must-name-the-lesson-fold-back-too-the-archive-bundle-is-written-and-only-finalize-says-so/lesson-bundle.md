# Lesson bundle — ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Ship must name the lesson fold-back too: the archive bundle is written and only finalize says so
- **Kind**: BugFix
- **Specs**: cmd_change
- **Paths**: src/commands/change.rs, specs/cmd_change/cmd_change.spec.md
- **Acceptance**: change ship names the lesson fold-back targets and the bundle path before the merge step, exactly as change finalize already does
- **Acceptance**: a change owning no specs gets guidance byte-identical to before the fix, so the prefix cannot leak into cases with nothing to fold
- **Acceptance**: the sibling do-not-merge blocker survives the fold-back prefix rather than being displaced by it
- **Acceptance**: the shared guidance is one pure function pinned by tests, not a string duplicated per verb

## Evidence

- Verification commit: `bfd3ef6224b2b311890ad9299a5d9bb3c8e371a0`
- Base commit: `fb88b9acaafe99abd83a637876331e83330e49fb`
- Verified by: `cargo test commands::change::`

## From the change's context.md

# Context

The lessons loop shipped in #697 with its third stage half-wired. `finalize_change` writes
`lesson-bundle.md` into the archive correctly, and the `finalize` command names the fold-back in
its `next_action`. `ship` does not — it builds its own next-action string.

`ship` is the verb the tool recommends. `ship-status` says "run `specsync change ship <id>`", and
the ship stages name it as the archive step. So on the primary path the bundle was assembled and
nothing said it existed.

That is the exact failure the lessons loop was built to end — knowledge produced where nobody
looks — reproduced inside the loop, on its own recommended path, in the same change that fixed it
everywhere else. It was found by running `ship` for real on #697 and reading the output rather
than assuming the stage worked because `finalize` did.

## Why it happened

Each lifecycle verb composes its own next-action prose. `finalize` and `ship` both end at "the
change is archived, now merge", and each wrote that sentence separately. Adding the fold-back to
one did not touch the other, and nothing pins them together.

This is the same shape as #687 (`merge_before_finalize_warning`, extracted for the same reason)
and as the two selections of "the current change" that disagreed in #697's own review.

## Already ruled out

**Calling `lessons_next_action` from ship.** It re-loads the change to find its modules and
returns a whole sentence including "merge the PR on GitHub", which ship must not say — ship's
tail depends on `--push`/`--wait` and on sibling changes. Reusing it would have produced two
merge instructions in one line.

## From the change's design.md

# Design

`ship_next_action(push, wait, siblings_before, fold_targets, bundle) -> String`, pure.

The existing push/wait/siblings matrix is preserved exactly and becomes the *tail*. When
`fold_targets` is non-empty, the fold-back instruction is prepended:

    write lessons into <targets> from <bundle>, then <existing guidance>

Empty targets return the tail unchanged — the control case, asserted directly.

## Why pure

The regression this guards is not "the code is wrong today". It is a future edit to one verb's
guidance that forgets the other, which is exactly how the defect arose. A pure function with
tests makes the coupling structural instead of remembered.

## Why not reuse `lessons_next_action`

It ends in "then merge the PR on GitHub". Ship's tail is conditional on `--push`, `--wait`, and
sibling changes, so reuse would emit two different merge instructions in one sentence. The two
verbs share the fold-back CLAUSE, not the whole sentence.

## From the change's testing.md

# Testing

`ship_names_the_lesson_fold_back_before_the_merge` — the regression. Asserts the string STARTS
with the fold-back, names the bundle, and still carries the merge instruction.

`ship_guidance_is_unchanged_when_there_is_nothing_to_fold` — **honest label: this is the CONTROL,
not a discriminator.** It passes before and after the fix. Its job is to prove the prefix cannot
leak into a change with no lessons to fold, across all three push/wait combinations.

`ship_keeps_the_sibling_blocker_alongside_the_fold_back` — the likeliest way to get this wrong is
to replace the tail rather than prepend to it, silently dropping "do not merge while any change
is active". This asserts both survive together.

## How the defect was found

By running `ship` on #697 and reading its output, not by reasoning about the code. The unit
suite passed throughout: nothing tested which verb emits which guidance. That is the coverage gap
this change closes.

## Where these lessons go

- `specs/cmd_change/context.md`
