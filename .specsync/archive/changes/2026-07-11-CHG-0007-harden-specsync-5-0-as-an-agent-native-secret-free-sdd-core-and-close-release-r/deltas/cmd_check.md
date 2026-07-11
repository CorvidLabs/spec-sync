## ADDED

### REQUIREMENT REQ-cmd-check-002
The check and fix pipeline SHALL remain deterministic and local.

Acceptance Criteria
- `--fix` performs deterministic markdown repairs and never invokes an embedded model or shell AI command.
- Requirements drift remains visible as validation guidance for a coding agent to resolve.
- Existing cache, enforcement, lifecycle, output-format, backup, and dry-run behavior remains intact.
