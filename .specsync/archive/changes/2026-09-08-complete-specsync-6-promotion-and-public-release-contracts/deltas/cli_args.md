## ADDED

### REQUIREMENT REQ-cli-args-015

The CLI help SHALL describe scoped review as a recorded human review with a stable reviewer claim, without claiming that local review recording or finalization authenticates the reviewer identity or retrieves an authenticated GitHub verdict.

Acceptance Criteria
- Reviewer help distinguishes a recorded claim from authentication enforced by separately configured hosted policy.
- The scope approver may perform the scoped human review.
- Argument grammar, pass/block behavior, and required review/freshness gates remain unchanged.
