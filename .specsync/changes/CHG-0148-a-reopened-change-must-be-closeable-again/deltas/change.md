## ADDED

### REQUIREMENT REQ-change-073

Scoped review evidence SHALL be permitted to move between a change's active workspace and its archive in either direction, and SHALL be refused anywhere else.

Acceptance Criteria
- A change that was finalized, reopened, re-checked and re-reviewed can be finalized again, leaving exactly one archive package and no active workspace.
- The move performed by reopen is accepted on the same terms as the move performed by finalize, since both relocate the same evidence between the only two locations a change occupies.
- Relocation to any other path is still refused, so the check continues to detect evidence moved outside the lifecycle.
