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
