## ADDED

### REQUIREMENT REQ-cli-args-022

The CLI SHALL parse `change audit --archives` as an opt-in request to also re-validate every archived change's own workflow-version history. The global `--strict` flag does not select it.

#### Acceptance Criteria

- Help lists `--archives` under `change audit`.
- `change audit` without `--archives` parses exactly as before, with or without `--strict`.
