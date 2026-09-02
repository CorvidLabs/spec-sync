## MODIFIED

### SPEC SECTION Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only: evidence completeness then in-process spec↔code sync. It does not spawn project tests or rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact. The rule covers the scoped review as well as the verification: readiness asks whether the recorded review is current, reports that answer as `current`, `stale`, or `unavailable`, and treats only `current` as satisfied. An unavailable guarantee reported as a satisfied one is worse than the refusal it conceals, and readiness that never asks receives no negative answer and reads its own silence as a pass.
7. The lessons loop surfaces at each of the three moments a lesson exists: `change new` names every affected module's `specs/<module>/context.md` that holds substantive prose, a FAILED `change check` names where to record what the failure taught, and BOTH `change finalize` and `change ship` name folding the archived bundle into those specs before their remaining guidance. Every surface is a pointer, never a dump, and none can fail a lifecycle command. A passing `change check` says nothing, and a change owning no affected specs receives the same guidance it received before the fold-back existed. Both verbs also emit a `lesson_bundle` path in `--json`.
8. `status`, `show`, a passing `check`, `approve`, `review`, `finalize`, and ship's finalize each end their text result with exactly one `Handoff:` line — after `Next:` where one is printed — reading `safe`, `conditional`, or `not yet`, an em-dash, the domain's reason, and, when readiness is not safe, `Before clearing:` followed by the domain's steps. The line renders the domain's `HandoffSummary` verbatim: the adapter never decides readiness itself and never prints a digest on it. JSON carries the same object under `summary.handoff` wherever a change summary is rendered and under `handoff` on the approve transition.

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
- Status requests renewed approval only for a material stable-scope change and lists each added or
  removed criterion, affected spec/path, dependency, supersession obligation, or changed intent in
  plain language; execution/evidence-only changes direct the user to `change check` instead.
- Newcomer output teaches `new → approve → implement → check → review → finalize → GitHub merge`.
- Agent-authored status identifies a missing scoped review and explains that opening or updating the
  PR requests the configured review check.
- Status names any strict validators required by `--strict`, project policy, or release/security
  classification without presenting a different lifecycle.
- `finalize` reports the implementation parent, archived path, finalization digest, and readiness
  for GitHub merge; it never claims to merge or invokes a provider merge API.
- JSON and text expose the same current gate and next action.
- Review output exposes the persisted `pass` or `block` verdict; domain rejection of a scope
  approver acting as reviewer exits non-zero without success output.
- Review identities are stable ASCII claims, attempts are append-only, and output does not imply
  authentication before the required GitHub check is proven.
