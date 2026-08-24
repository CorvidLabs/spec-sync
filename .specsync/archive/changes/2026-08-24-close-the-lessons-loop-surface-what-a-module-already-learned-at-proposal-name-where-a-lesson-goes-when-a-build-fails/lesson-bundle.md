# Lesson bundle — close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize
- **Kind**: Feature
- **Specs**: change, cmd_change, generator
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/generator.rs, specs/change/change.spec.md, specs/cmd_change/cmd_change.spec.md, specs/generator/generator.spec.md
- **Acceptance**: change new names each affected module context.md with its substantive line count, counting only authored prose and never a generated scaffold
- **Acceptance**: a FAILED change check names where the lesson goes, including for a bare check with no id; a passing check prints nothing
- **Acceptance**: finalize writes lesson-bundle.md into the archive durably and next_action names the fold-back targets before the merge
- **Acceptance**: lessons policy lives in src/change.rs and the generator owns what a scaffold looks like; the command layer only renders
- **Acceptance**: frontmatter stripping has one definition matching view::strip_frontmatter, pinned by a discriminating multi-rule regression test

## Evidence

- Verification commit: `9b905e965e057a9a8b88de0f2153db42859c67dc`
- Base commit: `ffbcf524b4847c5cebbed107975849b6427af324`
- Verified by: `cargo test change::`, `cargo test commands::change::`, `ruby --version`, `cargo test`, `python3 -S .github/scripts/validate-release-version.py`, `python3 -S .github/scripts/validate-workflow-runtime-pins.py`

## From the change's context.md

# Context

Lessons were already being written into `specs/<module>/context.md` — that is where a module's
accumulated knowledge is supposed to live, precisely so the next change to that module can read
it. Nothing surfaced them. They accumulated where nobody looked, which is indistinguishable from
not having written them.

The gap is that a lesson exists at three separate moments and the tool was silent at all three:

1. **Proposal** — the author is scoping a change to a module that has already taught someone
   something. That knowledge is on disk and unread.
2. **Build** — a verification just failed. An approach was tried and did not work. This is the
   only moment the dead end is fresh, and it is precisely when nobody stops to write it down.
3. **Archive** — the change is about to become inert history. This is the last moment its
   knowledge can be moved somewhere that is still read.

## Already ruled out

**SpecSync writing the lessons itself.** It would have to shell out to a particular agent, and it
does not need to: whoever just ran `finalize` is right there. `next_action` is the mechanism the
lifecycle already uses everywhere, and drill 032 confirms agents follow it to termination.

**Dumping context at proposal time.** A wall of text at creation gets scrolled past. Naming the
file with its size makes reading it a choice the author knows they are making.

**Nudging on every check.** A hint on a green check is noise, and there is usually nothing to
record. Only failure earns the interruption.

## What the accumulated lessons changed about this change

This change was scoped after reading the two context files its own feature surfaced, which is the
first real test of whether the loop is worth anything.

`specs/cmd_change/context.md` states, three separate ways, that the command layer holds no
lifecycle policy and that all policy lives in `src/change.rs`. The first implementation put the
lessons policy — reading context files, deciding what counts as substantive prose — in
`src/commands/change.rs`. That is policy in the command layer, and it violated the module's
stated architectural invariant.

The policy was moved to `src/change.rs` (`accumulated_lessons`, `lesson_fold_targets`,
`module_context_path`) and the command layer reduced to rendering. The loop caught a design flaw
in the change that adds the loop.

## What the independent review caught, and what it cost

An independent reviewer BLOCKED the first shippable version. Both blockers were the loop's own
failure mode aimed back at it.

**The scaffold detection was false.** The code documented "scaffold prompts are HTML comments".
The real `CONTEXT_TEMPLATE` is plain bullets, so every untouched `specs/<module>/context.md`
counted as four lines of knowledge. Stage 1 would have pointed every new adopter at a file that
had learned nothing — the exact behaviour the design doc says kills the surface. It survived
dogfooding because all 62 specs in this repository already have prose, so no untouched scaffold
existed here to trip over. `validator.rs` already enumerated those bullet strings; the fix asks
the generator instead, so the definition lives with the template rather than beside it.

**A delta silently deleted three unrelated documented behaviours.** The delta declared five
Behavioral Examples scenarios; the living spec ended up with one. `### Scenario:` in the section
body collides with the delta format's own `### SPEC SECTION` level. `change.spec.md` uses bold
scenarios and was unharmed; `cmd_change.spec.md` uses `###` and lost two-thirds of its section.
Filed as #699. The corruption depends on the TARGET file's convention, and 59 of 62 spec files
use the vulnerable style.

**Two tests proved nothing.** One fixture used a string the product never emits. One was labelled
a discriminator but is an invariant: with a single `---`, the old `split("---").nth(2)` also
returned the whole string. Both corrected, the second with an honest label and a second rule.

The lesson worth carrying: **dogfooding on a mature repository cannot see a new-adopter defect.**
Every affordance keyed on "has the author written anything yet" needs a fixture that is the real
generated artifact, because the mature repository no longer contains one.

## From the change's design.md

# Design

## Three surfaces, one rule

SpecSync assembles material and names the next step. It never writes a lesson and never blocks on
one. Every surface below is a pointer.

| moment | surface | silent when |
|---|---|---|
| proposal | `change new` lists module context files with line counts | no module has substantive prose |
| build | failed `change check` names the change's own `context.md` | the check passed |
| archive | `finalize` writes `lesson-bundle.md`, `next_action` names the fold | no affected specs |

## Layering

Policy in `src/change.rs`:

- `module_context_path(module)` — one definition of the convention, so surfacing and folding can
  never disagree about which file they mean
- `accumulated_lessons(root, modules)` — what counts as a lesson
- `lesson_fold_targets(root, id)` — where this change's lessons go
- `write_lesson_bundle(root, record, archive)` — assembly, best-effort

`src/commands/change.rs` renders these and decides nothing. This follows the thin-dispatch rule
already stated in `specs/cmd_change/context.md`, which the first draft of this change broke.

## Failure posture

Every path fails open. An unreadable context file yields no entry rather than an error; an
unwritable bundle leaves a successful archive intact. The alternative — a lifecycle command
failing over an authoring affordance — trades a real guarantee for a nicety.

This is deliberately the opposite posture from evidence validation, which fails closed. The
distinction is whether the artifact is *load-bearing for trust* (fail closed) or *an aid to the
author* (fail open). Lessons are the latter.

## Frontmatter

Frontmatter ends at its **closing** delimiter, never at the next `---` in the document. `---` is a
legal Markdown horizontal rule, so `split("---").nth(2)` truncates any body containing one — and
truncated material in a lesson bundle is indistinguishable from material nobody wrote. The helper
matches `view::strip_frontmatter` byte-for-byte, including BOM handling, so #696 can unify them
without a behaviour change.

## From the change's testing.md

# Testing

## Unit

- `strip_frontmatter_keeps_a_body_whose_horizontal_rule_is_not_frontmatter` — the truncation bug.
  A body with a rule and no frontmatter must survive whole.
- `strip_frontmatter_removes_real_frontmatter_and_keeps_later_rules` — the complementary case:
  real frontmatter goes, a later rule and everything after it stays. This test **failed on the
  first implementation** and is why the helper is delimiter-based rather than split-based.
- `accumulated_lessons_ignores_a_context_holding_only_scaffold` — a fresh scaffold must not
  advertise itself as knowledge.
- `accumulated_lessons_counts_substantive_prose_and_skips_absent_modules`.

## Dogfooded

`change new` for this very change surfaced its own modules' lessons:

```
Lessons: what these modules already learned:
  specs/change/context.md (101 line(s)) — read before scoping this change
  specs/cmd_change/context.md (21 line(s)) — read before scoping this change
```

Reading them changed the design (see `context.md`). That is the loop working end to end on
itself, not a demonstration arranged to succeed.

## Not covered

Whether an agent actually *writes* a good lesson when prompted. Out of reach of the suite; drill
032 covers `next_action` adherence generally.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/generator/context.md`
