## MODIFIED

### REQUIREMENT REQ-cmd-change-005

The change command adapter SHALL guide every user through the single one-approval workflow and
same-PR finalization without performing an external merge.

Acceptance Criteria

- `status` always prints exactly one explicit next action.
- `status`, `show`, a passing `check`, `approve`, `review`, `finalize`, and ship's finalize each end
  with exactly one `Handoff:` line (after `Next:` where one is printed) — `safe`, `conditional`, or
  `not yet`, the domain's reason, and `Before clearing:` steps when it is not safe. JSON carries the
  same decision under `summary.handoff` wherever a summary is rendered and under `handoff` on the
  approve transition. The adapter renders the domain's verdict and never computes its own.
- When scope approval is missing or stale, status prints the exact current digest next to that
  approval action.
- Status requests renewed approval for a material stable-scope change and lists each added or
  removed criterion, affected spec/path, dependency, supersession obligation, or changed intent in
  plain language, and for a semantic delta body edited after the approval that recorded it, naming
  each such module; other execution/evidence-only changes direct the user to `change check` instead.
- Newcomer output teaches `new → approve → implement → check → review → finalize → GitHub merge`.
- Agent-authored status identifies a missing scoped review and explains that opening or updating the
  PR requests the configured review check.
- Status names any strict validators required by `--strict`, project policy, or release/security
  classification without presenting a different lifecycle.
- `finalize` reports the implementation parent, archived path, finalization digest, and readiness
  for GitHub merge; it never claims to merge or invokes a provider merge API.
- JSON and text expose the same current gate and next action; text status names re-approval for
  every approved, implementing, or verifying change whose approval is not current.
- Review output exposes the persisted `pass` or `block` verdict; domain rejection of a scope
  approver acting as reviewer exits non-zero without success output.
- Review identities are stable ASCII claims, attempts are append-only, and output does not imply
  authentication before the required GitHub check is proven.
