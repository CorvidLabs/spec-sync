## ADDED

### REQUIREMENT REQ-change-127

The system SHALL refuse `change review` and `change finalize`, and SHALL NOT report readiness `safe` from `change status` or `change ship-status`, for a change whose record is not archived while the remote default branch holds an archive of the same change id that `HEAD` does not contain. The refusal SHALL name the reference and commit of that archive and the next step: merge or rebase onto that reference. An archive that `HEAD` already contains SHALL NOT be refused: that is the reopen-and-rearchive path.

Acceptance Criteria
- With the remote default holding an archive of the change, `change review` refuses before recording acceptance and the record stays `verifying`.
- `change finalize` refuses with the same named cause and writes no archive.
- Without an upstream archive, a squash before finalize still reviews and finalizes.
- An unarchived record whose archive `HEAD` already contains is not refused.
