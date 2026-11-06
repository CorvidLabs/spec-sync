---
change: report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it
artifact: requirements
---

# Requirements

REQ-change-093 (modified, change): an archive whose terminal evidence is `corrupt-history` is `not-yet` and names the restore; only an archive that authenticates is `safe`; `handoff_summary` uses the same terminal-evidence verdict as `ChangeSummary`.

REQ-cmd-change-014 (modified, cmd_change): outside the shipping window an authenticated archive is told there is no further action, and an archive that fails its integrity check is told the restore that recovers it.

No new requirement IDs.
