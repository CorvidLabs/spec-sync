---
change: refuse-a-broken-policy-and-commit-the-archive-move
artifact: requirements
---

# Requirements

Opening a change fails when `.specsync/sdd.json` cannot be parsed, and `change status` fails with that same error instead of printing a briefing. `specsync rules` warns when the named principles file is not on disk and still exits 0. `change finalize` commits the archive move as `chore(lifecycle): archive <id>`.

The normative statements are REQ-change-108, REQ-cmd-change-021, REQ-cmd-change-022, and REQ-cmd-rules-003.
