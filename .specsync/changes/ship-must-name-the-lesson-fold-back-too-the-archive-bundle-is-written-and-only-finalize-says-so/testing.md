---
change: ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so
artifact: testing
---

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
