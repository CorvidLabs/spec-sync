---
change: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
artifact: requirements
---

# Requirements

## REQ-change-050: usable defaults and tolerant active-change discovery

A newly initialised project either detects a verification command or is told at
init time that it must supply one. Active-change discovery treats a directory with
no state.json as not an active change here rather than as corruption. Verification
exposes a lock-free body so a lock holder can re-run it without deadlocking.
