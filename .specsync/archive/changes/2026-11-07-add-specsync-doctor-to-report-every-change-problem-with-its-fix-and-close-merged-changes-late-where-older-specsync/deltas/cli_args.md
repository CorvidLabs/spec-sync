## ADDED

### REQUIREMENT REQ-cli-args-025

The `doctor` subcommand SHALL accept `--fix`, `--dry-run`, a repeatable `--close-late <id|all>`, `--actor <name>` and `--reason <text>`, together with the global output flags; argument parsing SHALL refuse `--close-late` given without `--fix`, `--actor` and `--reason`, and `--dry-run` given without `--fix`.

Acceptance Criteria
- `specsync doctor`, `specsync doctor --fix`, `specsync doctor --fix --dry-run` and `specsync doctor --fix --close-late a --close-late b --actor X --reason Y` parse.
- `specsync doctor --close-late a --actor X --reason Y` (no `--fix`), `specsync doctor --fix --close-late a` (no actor or reason) and `specsync doctor --dry-run` are refused by argument parsing.
