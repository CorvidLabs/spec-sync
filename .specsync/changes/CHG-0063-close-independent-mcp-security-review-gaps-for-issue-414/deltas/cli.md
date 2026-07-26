## ADDED

### REQUIREMENT REQ-cli-006

The root CLI dispatcher SHALL preserve the user-requested root spelling for commands whose
retained-capability engines must detect public path replacement.

Acceptance Criteria

- MCP, check, coverage, generate, score, report, and comment receive the validated requested path
  without eager canonicalization.
- Generate retains the bound root authority through publication so a redirect after checked
  coverage returns cannot redirect an output write into the replacement.
- Other commands retain their established canonical-root dispatch behavior.
- A symlink/junction replacement after capability retention remains observable to checked
  traversal and cannot be hidden by dispatcher canonicalization.
