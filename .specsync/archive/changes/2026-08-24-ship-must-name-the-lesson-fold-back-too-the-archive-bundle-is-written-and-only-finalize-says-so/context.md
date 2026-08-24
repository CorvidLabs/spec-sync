---
change: ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so
artifact: context
---

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
