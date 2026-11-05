## ADDED

### REQUIREMENT REQ-cmd-change-026

`specsync change send-back` SHALL delegate to the change module, render the persisted change and send-back record as deterministic JSON under `--json`, and in text say the approval is cancelled and name answering the scope questions again and `change approve` as the next steps.

#### Acceptance Criteria

- Text output names the change, says its approval is cancelled, and names `change answer` and `change approve`.
- JSON output carries `change` and `audit`.
- A refused send-back exits non-zero with the domain error.
