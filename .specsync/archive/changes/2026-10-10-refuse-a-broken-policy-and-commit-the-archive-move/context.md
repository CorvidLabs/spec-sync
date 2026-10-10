---
change: refuse-a-broken-policy-and-commit-the-archive-move
artifact: context
---

# Context

Three review notes were left until after the optional module 3md file landed. A broken `.specsync/sdd.json` was swallowed by `change new` and `change status`, so a damaged policy looked like a project with no principles. `specsync rules` named a principles file that was not on disk and said nothing was wrong. `change finalize` moved the package to the archive and left that move uncommitted, so the next checkout could lose it. `change ship` already committed the same move.

The constraint this change honors is that a broken policy must not look like no principles file. SpecSync still does not copy lessons into the principles file, and it still does not merge the pull request.
