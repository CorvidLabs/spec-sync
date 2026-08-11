## MODIFIED

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
- Ordinary `change review <id> --reviewer <identity>` accepts a stable ASCII reviewer claim,
  defaults to `pass`, and retains independent-review semantics; `--verdict pass|block` records an
  explicit conclusion without adding another lifecycle mode or approval.
- `change review <id> --self-review --actor <scope-approver> --reason <reason>` is the only
  self-review grammar. It requires every listed audit input, rejects `--reviewer` in the same
  invocation, and is visibly described as an audited solo-maintainer exception rather than an
  independent review.
- Existing historical repair commands remain available without appearing in the newcomer core path.
- Existing change grammar remains compatible.
