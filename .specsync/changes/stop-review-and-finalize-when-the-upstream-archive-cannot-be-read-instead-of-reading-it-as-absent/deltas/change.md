## ADDED

### REQUIREMENT REQ-change-140

When a remote default exists and its archived state records cannot be listed or read, the upstream-archive lookup SHALL report that it cannot tell whether the change is already archived there, rather than reporting no upstream archive. `change review` and `change finalize` SHALL refuse before acceptance and write no archive, and `change status` and `change ship-status` SHALL report the same refusal. The refusal SHALL name the remote default reference and how to find out (SHIP-3.c).

Acceptance Criteria
- A failed listing, a failed or truncated batched read, or a failed addition or ancestry lookup for a matching package is reported as cannot-tell.
- No remote default, or a readable archive with no copy of the change, reports nothing.
