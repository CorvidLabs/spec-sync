## MODIFIED

### REQUIREMENT REQ-cmd-change-005

The change command adapter SHALL guide every user through the single one-approval workflow and
same-PR finalization without performing an external merge.

Acceptance Criteria

- `status` always prints exactly one explicit next action.
- When scope approval is missing or stale, status prints the exact current digest next to that
  approval action.
- Status requests renewed approval only for a material stable-scope change and lists each added or
  removed criterion, affected spec/path, dependency, supersession obligation, or changed intent in
  plain language; execution/evidence-only changes direct the user to `change check` instead.
- Newcomer output teaches `new → approve → implement → check → review → finalize → GitHub merge`.
- Agent-authored status identifies a missing scoped review and explains that opening or updating the
  PR requests the configured review check; after a valid self-review it names the audited exception
  rather than claiming independent hosted review.
- Status names any strict validators required by `--strict`, project policy, or release/security
  classification without presenting a different lifecycle.
- `finalize` reports the implementation parent, archived path, finalization digest, and readiness
  for GitHub merge; it never claims to merge or invokes a provider merge API.
- JSON and text expose the same current gate and next action.
- Review output exposes persisted mode and `pass` or `block` verdict. Ordinary review rejects a
  scope approver acting as reviewer; explicit self-review accepts only the matching scope approver
  with a non-empty reason, and all domain rejection exits non-zero without success output.
- Review identities are stable ASCII claims, attempts are append-only, and output does not imply
  hosted authentication for an audited self-review.
