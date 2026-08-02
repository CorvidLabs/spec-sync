## MODIFIED

### REQUIREMENT REQ-change-047

The change lifecycle SHALL prefer completing incomplete selected artifacts over definition approval for draft changes once the interview is complete and artifact completeness validation fails.

Acceptance Criteria

- When selected artifacts contain incomplete HTML TODO comment stubs or are empty, summarize_change sets artifacts_complete to false and next_action does not recommend change approve.
- After selected artifacts are complete, draft next action may recommend definition approval.

### REQUIREMENT REQ-change-048

The change lifecycle SHALL refuse definition approval when a semantic delta uses ADDED for a requirement ID whose requirement heading already exists in the living module requirements file, and the diagnostic SHALL steer agents to MODIFIED.

Acceptance Criteria

- validate_delta_files and approve_definition fail with cannot add existing block for living requirement IDs under ADDED.
- The error text mentions MODIFIED.
- MODIFIED of an existing living requirement ID validates successfully.
