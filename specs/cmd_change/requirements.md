---
spec: cmd_change.spec.md
---

# Requirements

### REQ-cmd-change-001

The system SHALL provide equivalent text and JSON interfaces for the complete SDD lifecycle and its explicit semantic-succession adoption.

Acceptance Criteria
- Humans receive concise next-action guidance and one consistent accepted-validity reason.
- Agents receive stable records, summaries, question identifiers, manifests, supersedes edges, and successor evidence.
- `change supersede` records explicit predecessor/path/module/digest obligations only before definition approval.
- `change approve --portable-5-0-1` records one atomic marked pair and renders it as one definition transition.
- Audited reopen returns the verifying change and its versioned supersession event in deterministic JSON.
- Active accepted check/status/reopen/archive eligibility render exact, successor-covered, or stale; archived status renders authenticated-history or corrupt-history.

### REQ-cmd-change-002

The change command adapter SHALL render accepted metadata correction and its complete effective audit
view equivalently in text and deterministic JSON.

Acceptance Criteria

- Correct JSON is the typed domain result containing original/effective values, ordered history,
  actor, reason, timestamp, digests, added artifacts, prior evidence, gate health, and next action.
- Human output names the field transition, newly required artifacts, and next required gate.
- Show and status expose corrected effective values and history rather than silently reporting the
  original answer as current.
- Domain failures exit non-zero and emit no success output.

### REQ-cmd-change-003

The change command adapter SHALL expose audited exact acceptance-owner correction equivalently in
text and deterministic JSON.

Acceptance Criteria

- `change correct-owner` delegates all ownership policy to the change domain module.
- JSON emits the persisted corrected change record, including its append-only owner correction.
- Human output names the exact path, canonical module, actor, and next definition-approval gate.
- Domain rejection exits non-zero without success output or partial lifecycle mutation.

### REQ-cmd-change-004

The change command adapter SHALL resolve batch correct-owner selection, delegate policy to the
change domain, and render text/JSON results without partial lifecycle mutation on failure.

Acceptance Criteria

- `change correct-owner` delegates all ownership and transactionality policy to the change domain.
- JSON emits the persisted corrected change record, including every appended owner correction.
- Human output names the number of corrections appended (or the single path/module for one entry)
  and the next definition-approval gate.
- Domain rejection exits non-zero without success output or partial lifecycle mutation.

### REQ-cmd-change-005

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

### REQ-cmd-change-check-scoped-001

`specsync change check` SHALL run scoped verification for one change and SHALL NOT
invoke full archive terminal-evidence revalidation.

When `--commit` is set, after a successful first verification the command SHALL
commit any materialized working-tree changes, re-run scoped verification against
the committed tip, and commit the resulting verification evidence. A failed first
verification SHALL leave the git history unchanged.

When `--push` is set without `--commit`, the command SHALL fail before running
verification. When both are set, a successful commit sequence SHALL end with
`git push`.

Acceptance Criteria

- `change check --commit` leaves recorded verification evidence that
  `change audit --strict` accepts for that change when the tree is otherwise clean.
- A failing first verification produces no new commits.
- `--push` without `--commit` fails with an error naming the requirement.

### REQ-cmd-change-audit-001

`specsync change audit` SHALL report active-workspace and living-spec integrity and exit non-zero when the report contains errors.

Acceptance Criteria
- Output does not dump authenticated-history lines for archived changes.
- Checked count reflects active changes in scope.

### REQ-cmd-change-006

The change command adapter SHALL render draft next-action guidance that prefers completing incomplete selected artifacts over recommending definition approval, using lightweight artifact completeness without digest-bearing loaders for text mode.

Acceptance Criteria

- Text and JSON next-action guidance do not recommend change approve for interview-complete drafts with incomplete selected artifacts.
- Completeness guidance remains available without writing digests into cleartext text sinks.

### REQ-cmd-change-007

`specsync change ship-status` SHALL report ship readiness for an active change,
including local tip classification and optional live GitHub check-run trust:

- HEAD tip class: `product`, `review_only`, `archive_only`, or `other`
- verification tip presence and ancestry relative to HEAD
- whether a scoped review is recorded
- trust guidance for staged product → review → archive tips
- when `GITHUB_TOKEN` is set and the git remote is GitHub, live check-run trust for the
  parent commit SHA (falling back to HEAD) with `trust.status` in
  `green` | `pending` | `failed` | `empty` | `unavailable` and
  `trust.source` = `github_check_runs`
- when the token is absent, lookup fails, or `SPECSYNC_SHIP_LOCAL_GUIDANCE` is set,
  `trust.status` is `local_guidance` or `unavailable` and the command still succeeds
- ordered ship stages with concrete next actions
- blockers, warnings (including merge-before-finalize), and `ship_next`
- sibling active change ids and multi-active ordering warnings when present

Acceptance Criteria

- JSON includes `tip_class`, `tip_sha`, `parent_sha`, `trust`, `stages`,
  `verification_present`, `verification_ancestor_of_head`, `review_present`,
  `ready_to_finalize`, `blockers`, `warnings`, `sibling_active_ids`, and `ship_next`.
- Tip class is derived from the paths changed in HEAD relative to its first parent
  (or working-tree vs HEAD when HEAD is not a useful tip).
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A non-archived change always warns not to merge while the change is still active.
- When other active changes exist, warnings name finalize-one-at-a-time, do-not-batch-reviews,
  and do-not-merge-with-active-changes rules.
- Live trust queries never fail the ship-status command; errors appear under `trust.error`
  with `trust.status` = `unavailable`.
- Offline and no-token runs keep `trust.status` = `local_guidance` without network access.

### REQ-cmd-change-008

`specsync change ship [ID]` SHALL run ship preflight for one change and, when
`ready_to_finalize` is true, perform finalize. When not ready it SHALL exit
non-zero and print blockers and the next stage without mutating state.

Optional orchestration flags after a successful finalize (or for already-archived
changes):

- `--push` SHALL commit the archive tip when the working tree is dirty and run
  `git push` for the current branch.
- `--wait` SHALL poll GitHub check-runs for HEAD (using the same in-process REST
  path as ship-status trust) until overall status is `green`, `failed`, timeout,
  or offline/`GITHUB_TOKEN` absent (reported as `local_guidance` without failing
  when no token).
- `--wait-timeout-secs` SHALL bound the wait (default 900).
- `--dry-run` SHALL refuse combination with `--push` or `--wait`.

Acceptance Criteria

- Exit code 0 only when preflight is clean and finalize succeeds (or the change
  is already archived and nothing remains), and optional push/wait succeed.
- Exit code non-zero when blockers remain, push fails, wait sees failed checks,
  or wait times out.
- Text and JSON outputs name the current tip class and next ship stage; JSON may
  include `push` and `wait` result objects when those flags are used.
- When sibling active changes remain after finalize, next guidance names them and
  requires their own check → review → ship cycle before merge.

### REQ-cmd-change-009

Text `specsync change show`, `specsync change status <id>`, and aggregate
`specsync change status` SHALL fail closed before emitting a successful lifecycle projection
when an active change has invalid correction-ledger health.

Acceptance Criteria

- Each affected text command exits non-zero and emits the same safe correction-ledger integrity
  diagnostic.
- No successful identity, answer, next-action, or correction-count output precedes that diagnostic.
- JSON inspection retains its typed fail-closed behavior.
- Valid active changes retain their existing text output and exit status.

### REQ-cmd-change-010

Lifecycle commands that mutate an existing change and then render its record SHALL validate
correction-ledger integrity before persistence and render from that validated transaction result.

Acceptance Criteria

- `answer`, `depend`, and `supersede` reject an invalid existing correction ledger before
  changing lifecycle files.
- A ledger that becomes invalid while a mutation waits for the project lock is rejected after lock
  acquisition and before persistence.
- Successful mutation output uses the effective definition, correction history, and selected
  normal/strict summary returned by the domain transaction, so a later ledger change cannot
  produce a nonzero exit or contradictory JSON after persistence.
- Read-only text show, status, and list views retain their existing fail-closed behavior.
- Valid mutation and rendering behavior is unchanged.
- The `cmd_change` canonical contract version is incremented.
