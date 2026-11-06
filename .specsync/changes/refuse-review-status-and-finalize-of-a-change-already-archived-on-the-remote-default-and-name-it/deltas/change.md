## ADDED

### REQUIREMENT REQ-change-127

The system SHALL refuse `change review` and `change finalize`, and SHALL NOT report readiness `safe` from `change status` or `change ship-status`, for a change whose id is already archived in reachable history (HEAD or the remote default) while its own record is not archived. The refusal SHALL name the reference and commit of that archive and the next step: merge or rebase onto that reference.

Acceptance Criteria
- With the remote default holding an archive of the change, `change review` refuses before recording acceptance and the record stays `verifying`.
- `change finalize` refuses with the same named cause and writes no archive.
- Without an upstream archive, a squash before finalize still reviews and finalizes.
