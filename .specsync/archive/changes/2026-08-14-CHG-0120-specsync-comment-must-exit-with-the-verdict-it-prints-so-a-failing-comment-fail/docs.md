---
change: CHG-0120-specsync-comment-must-exit-with-the-verdict-it-prints-so-a-failing-comment-fail
artifact: docs
---

# Docs

CHANGELOG entry under Unreleased → Fixed, stating the CI consequence rather than
the code change: as a CI step, `comment` was a permanent pass that posted its own
failure.

No README change is needed. The README already documents `comment` as a CI step
whose failure fails the build — the documentation described the intended
behaviour all along, and this change makes the command match it.
