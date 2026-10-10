---
change: refuse-a-broken-policy-and-commit-the-archive-move
artifact: docs
---

# Docs

`change finalize` commits the archive move. The workflow, quickstart, and CLI pages say to push that commit and wait for checks. They no longer tell the reader to commit the archive by hand.

`specsync rules` says a named principles file is missing when the path is not a file. The configuration page says a policy that cannot be parsed is an error, not "no principles file".
