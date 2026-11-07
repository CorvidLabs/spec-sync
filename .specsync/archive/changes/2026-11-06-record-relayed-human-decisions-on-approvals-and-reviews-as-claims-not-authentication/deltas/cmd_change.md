## ADDED

### REQUIREMENT REQ-cmd-change-relayed-decision-001

`change approve` and `change review` SHALL take an optional relayed decision through `--words`, `--where` and `--recorded-by`, recording `decided_by` from `--actor` or `--reviewer`. `change status <id>` and `change show <id>` SHALL show the effective approval's actor and the latest review's reviewer and verdict, each marked as a recorded claim, not authenticated, and any relayed decision beside the record it belongs to, followed by the shared boundary sentence. JSON output SHALL carry the same data under `recorded_claims`.

#### Acceptance Criteria

- Giving one or two of the three flags is refused before anything is written, naming every missing flag and the full command to re-run; re-running with all three records the decision.
- A relayed approval without `--actor` is refused before anything is written, naming `--actor <who decided>`.
- A relayed decision prints as `Relayed: <decided_by> said "<words>" (<where>); recorded by <recorded_by>; a relayed claim, not authenticated: it records who is said to have decided, not proof that they did.`
- Text output escapes control characters in recorded names and relayed fields, so recorded words cannot move the cursor or recolour the terminal.
- A change with no approval and no review prints neither line, and its JSON omits `recorded_claims`.
