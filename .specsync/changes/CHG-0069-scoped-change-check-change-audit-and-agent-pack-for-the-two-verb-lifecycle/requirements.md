---
change: CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle
artifact: requirements
---

# Requirements

### REQ-change-audit-project-001

The change module SHALL expose `audit_project` that validates active change workspaces and living
SDD policy/spec coherence without rewalking archived terminal evidence by default.

Acceptance Criteria

- `audit_project` does not load or re-authenticate every archived change's terminal evidence.
- `check_project` remains available for full integrity including archives (tests / rare callers).
- CLI project-health surface uses the active-only path.

### REQ-change-check-scoped-002

`check_change` SHALL continue to materialize approved deltas and run verification for one selected
change only; project-wide archive integrity is not part of that function.

Acceptance Criteria

- Selecting zero, one, or many open changes behaves as before (nothing / that id / error listing ids).
- Archive terminal evidence is not required for a successful scoped check.

### REQ-cmd-change-check-scoped-001

`specsync change check` SHALL run scoped verification for one change and SHALL NOT invoke full
archive terminal-evidence revalidation.

Acceptance Criteria

- Text success ends with a verified marker and a Next action when possible.
- JSON emits verification only (not a full project archive evidence dump).
- Failure exits non-zero with actionable Next guidance.

### REQ-cmd-change-audit-001

`specsync change audit` SHALL report active-workspace and living-spec integrity and exit non-zero
when the report contains errors.

Acceptance Criteria

- Output does not dump authenticated-history lines for archived changes.
- Checked count reflects active changes in scope.

### REQ-cli-change-audit-001

The CLI SHALL expose `specsync change audit` as a first-class `ChangeAction` alongside `change check`.

Acceptance Criteria

- Help text distinguishes scoped check from active-only audit.
- Parsing accepts `change audit` with no change id.

### REQ-agents-check-audit-commands-001

`specsync agents install` SHALL generate `/specsync:check` and `/specsync:audit` command files for
tools that support project-local commands, and skill prose SHALL teach the two-verb lifecycle model.

Acceptance Criteria

- Claude, Cursor, and Gemini receive check and audit command files.
- Skill content distinguishes `change check` (scoped) from `change audit` (actives + living specs).
- Template version advances so upgrades refresh generated artifacts.

### REQ-hooks-two-verb-001

Installed hooks instruction snippets SHALL document `change check` as scoped verification and
`change audit` as active-workspace project health, and SHALL NOT instruct agents to treat check as
full archive terminal-evidence validation.

Acceptance Criteria

- Claude/Agents.md-style snippets mention both verbs.
- Cursor/Copilot snippets mention the two-verb distinction.

### REQ-commands-change-audit-dispatch-001

The change command dispatcher SHALL route `Audit` to active-only project audit and `Check` to scoped
verification without dual-wiring full archive integrity into check.

Acceptance Criteria

- Check path does not call full archive integrity.
- Audit path fails closed on active/living-spec errors only.
