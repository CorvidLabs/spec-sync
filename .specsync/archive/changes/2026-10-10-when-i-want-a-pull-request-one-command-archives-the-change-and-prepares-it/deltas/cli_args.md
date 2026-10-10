## ADDED

### REQUIREMENT REQ-cli-args-019

The CLI SHALL accept `specsync change pr` as the command that archives a ready change, pushes the branch, and prepares the pull request. The command SHALL NOT merge the pull request.

Acceptance Criteria
- `specsync change pr` parses as a change subcommand.
- The command takes an optional change id.
- Parsing it does not imply a merge.
