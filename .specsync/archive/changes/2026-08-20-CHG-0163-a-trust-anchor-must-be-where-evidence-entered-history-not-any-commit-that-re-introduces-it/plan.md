# Plan

1. Add an archive-introduction index: `git log --diff-filter=A --no-renames` over the archive
   root, resolving each addition's change ID from the committed `state.json`. Cache per
   `(root, archive-root, resolved HEAD, resolved remote-default)`.
2. `admissible_archive_introductions`: keep introductions no strictly earlier introduction of the
   same change supersedes, qualified by the reopen-ledger generation.
3. Bound all four evidence stages by that set, not only the archived one.
4. Tests: three laundering shapes refused, an honest relocation still authenticated, and a
   gitignored stray file ignored.
5. Sample one archive per risk class against the pre-fix baseline.
6. Widen the sandbox drill to cover the vector that involves no rename.
