---
change: one-canonical-frontmatter-reader-for-crlf-checkouts
artifact: plan
---

# Plan

Steps 1–3 of the #696 migration order are described as independently shippable, and they are. They
are landed together anyway, for one reason: step 1 alone leaves `view::strip_frontmatter` and
`change::strip_yaml_frontmatter` in place, and shipping a fix while a parallel implementation
survives is the exact failure `src/change_tests.rs` records as having happened seven times in this
release. Splitting here would ship the eighth on purpose.

Steps 4–6 are **not** landed, and are named in `tasks.md` so they read as decisions rather than
omissions.

## Order of work

1. Normalize inside `parse_frontmatter`. This alone fixes `specsync view` and 17 other
   unnormalized call sites, and it is the step that closes the shipped bug.
2. Promote the correct stripper into `parser`, delete the two wrong ones, repoint their callers.
3. Pin `.specsync/**/*.md` in `.gitattributes`.
4. For each test, splice the pre-change implementation back in and confirm the test goes red.
   A CRLF fixture that passes before the change proves nothing, and every fixture in this
   repository was LF precisely because nobody checked that.
5. `cargo fmt`; `cargo clippy -- -D warnings` (bare — `--all-targets` has pre-existing failures in
   test code, and clippy is not in the project's verification commands, so `change check` will not
   catch a clippy failure).
6. Semantic deltas, then `change approve`, `change check`, `change audit --strict`.

## Known state of the branch

There is a second, pre-existing active change on `main`
(`say-how-the-lesson-fold-back-terminates-and-that-ship-names-it-too`, `verifying`) whose PR was
merged without finalize. It is not touched by this change, and its presence means multi-active
ordering warnings are expected from `audit`. Finalization order is a separate decision for
whoever ships these.
