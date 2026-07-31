## ADDED

### REQUIREMENT REQ-cli-args-009

The shared CLI grammar SHALL expose the single discoverable change workflow without a SpecSync
merge command or lifecycle-mode selection.

Acceptance Criteria

- `change new`, `change approve`, `change check`, `change status`, and `change finalize` use plain
  names and help text matching the documented path.
- Existing global `--strict` selects additional validators on the same commands and evidence; it
  does not select another lifecycle.
- No lifecycle-mode, second-approval, closing-approval, `finalize-merge`, or SpecSync `merge`
  grammar is added.
- `change finalize <id>` prepares and archives the current PR change but has no GitHub merge input.
- `change review <id> --reviewer <identity>` accepts a stable ASCII reviewer claim and defaults to
  `pass`; optional `--verdict pass|block` records an explicit conclusion.
- Existing historical repair commands remain available without appearing in the newcomer core path.
- Existing change grammar remains compatible.
