---
change: change-approve-has-a-dry-run-that-reports-every-check-s-first-refusal-and-writes-nothing
artifact: delta
module: cli_args
---

## ADDED

### REQUIREMENT REQ-cli-args-027

The `change approve` subcommand SHALL accept `--dry-run` together with its existing flags and the global output flags.

Acceptance Criteria
- `specsync change approve <id> --dry-run` and `specsync --json change approve <id> --dry-run --actor Ada` parse; `specsync change approve <id>` still parses, with the dry run off.
