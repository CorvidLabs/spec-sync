## ADDED

### REQUIREMENT REQ-cli-args-relayed-decision-001

The `change approve` and `change review` grammar SHALL accept optional `--words`, `--where` and `--recorded-by` strings, and the `--actor` and `--reviewer` help SHALL say that the name is a recorded claim, not authenticated identity.

#### Acceptance Criteria

- `change approve <id> --actor <name> --words <text> --where <text> --recorded-by <agent>` and the same three flags on `change review <id> --reviewer <name>` parse into the change action.
- `change approve` and `change review` without the three flags parse exactly as before.
