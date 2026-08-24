---
change: close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails
artifact: context
---

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
