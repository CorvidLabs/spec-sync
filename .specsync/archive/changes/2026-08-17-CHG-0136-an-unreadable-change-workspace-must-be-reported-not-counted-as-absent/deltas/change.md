## ADDED

### REQUIREMENT REQ-change-068

Enumerating active changes SHALL return what could be read and what could not, as separate facts, so that no caller can mistake an unreadable workspace for an absent one.

Acceptance Criteria
- The roster reports readable records and unreadable workspaces separately, and each unreadable entry carries the workspace identity and a reason naming the offending path.
- A workspace that cannot be read does not abort enumeration: its healthy siblings are still returned.
- A failure that leaves no partial truth to report — the changes directory itself being unreadable — remains a hard error rather than an empty roster.
- A directory with no state file is still skipped rather than reported unreadable, because a husk left by a branch switch is not an active change here.
- The plain record list used by digest, ledger and successor computations continues to fail closed on any unreadable workspace, since a silently short roster is worse there than a hard error.
- A project with no active changes still yields an empty roster with nothing unreadable.
