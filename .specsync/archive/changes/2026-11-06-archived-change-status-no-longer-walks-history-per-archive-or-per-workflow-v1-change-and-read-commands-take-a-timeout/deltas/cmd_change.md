## ADDED

### REQUIREMENT REQ-cmd-change-032

`change status`, `change show`, `change list` and `change ship-status` SHALL accept `--timeout <secs>`, and SHALL read `SPECSYNC_TIMEOUT` when the flag is absent. With neither, there SHALL be no deadline, as before. When the deadline passes, the command SHALL exit 124. Text output SHALL name the command, the elapsed limit, the phase and progress, that nothing was written, and the exact resume command. `--json` output SHALL be `{"error":"timeout","phase":...,"progress":...,"wrote":false,"resume":...}`.

Acceptance Criteria
- `change status <id> --timeout 0 --json` on an archived change exits 124 with the JSON shape above and changes no file.
- Rerunning the printed resume command without a timeout gives the same verdict as a run that never timed out.
- With no flag and no environment variable, behaviour and output are unchanged.
