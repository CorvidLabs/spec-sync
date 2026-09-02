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
- whether a scoped review is recorded, and whether the recorded review is still current
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
  `review_currency`, `ready_to_finalize`, `blockers`, `warnings`, `sibling_active_ids`,
  and `ship_next`.
- `review_currency` is `missing`, `unreadable`, `current`, `stale`, or `unavailable`, and
  `ready_to_finalize` is true only when it is `current`. Existence of `review.json` is not
  currency: finalization additionally requires the recorded review to still match the tree, so
  readiness that asked only whether the file existed recommended the very verb that then refused.
- A recorded review that is decidably out of date is a blocker naming what moved and the
  re-review that repairs it, and the review stage reports `current` rather than `done` so the
  named next action is the recovery instead of the refused verb.
- A recorded review whose currency could not be determined is reported as `unavailable` and is
  never reported as satisfied. It produces a warning stating that currency could not be
  determined and naming the re-review that re-anchors it, rather than a blocker, because whether
  an unobtainable guarantee ought to block is not a question this command settles.
- `ship-status` and `finalize` reach the same conclusion about the same change on the same tree:
  ship-status never reports a change ready that finalization will refuse on review currency.
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

### REQ-cmd-change-011

Commands that enumerate active changes SHALL distinguish an empty project from a project whose workspaces could not be read, and SHALL never present a partial roster as a complete one.

Acceptance Criteria
- `list` and `status` print every readable change, then name each unreadable workspace with the reason including the offending path, and exit non-zero.
- The empty-project line is printed only when enumeration succeeded and found nothing, so a project whose only workspace is unreadable never reports itself as empty.
- `ship-status` reports the same roster and the same non-zero exit for the same tree, and its JSON carries the unreadable entries alongside the readable ones.
- JSON output is a single parseable document in both cases: the historical bare array while every workspace is readable, and an object carrying `changes` and `unreadable` once any workspace is not.
- `ship` and lifecycle commit resolution refuse to infer a target change while any workspace is unreadable, rather than selecting from the readable remainder.
- Sibling-active-change reporting counts unreadable workspaces as active, so an unreadable workspace is never reported as nothing else being in flight.
- A project with no active changes retains its existing empty-project output and zero exit status.

### REQ-cmd-change-012

Commands that stage the whole worktree SHALL apply the sequence-ledger floor before staging, and SHALL NOT block the author when they do.

Acceptance Criteria
- Materialize, verification-evidence and archive commits all floor the ledger before `git add -A`.
- A change whose ledger went stale while its branch sat still completes, because the author caused nothing and blocking them would punish a race they cannot observe.
- The disclosure appears on standard error rather than standard output, so `--format json` output remains a single parseable document.

### REQ-cmd-change-013

The lifecycle staging path SHALL raise a stale sequence ledger before staging it, and that wiring SHALL be asserted rather than only the function that performs it.

Acceptance Criteria
- A commit produced by the lifecycle staging path over a working tree whose ledger is below the committed mark carries the committed mark, not the stale one.
- Removing the raise from the staging path fails a test, so the connection between the mechanism and its caller cannot be severed while the suite stays green.

### REQ-cmd-change-014

`change ship-status` SHALL name a next action the same binary will accept, and SHALL resolve a change's verification and review evidence from wherever that change currently lives.

Acceptance Criteria
- Outside the shipping window — draft, accepted, and archived — the printed next action equals the lifecycle next action, so a draft is told to answer its interview rather than to commit verification, and an archived change is told there is no further action.
- The next action is always a runnable command and never a restatement of a blocker; blockers continue to render on their own lines.
- An archived change reports the verification commit and the scoped review recorded in its archive package, rather than reporting none and missing because the artifacts were sought at the active workspace path it has left.
- Evidence resolution reuses the single active-or-archive workspace resolver rather than introducing a third path-construction idiom.
- An unreadable or unparseable archived verification artifact reports no verification evidence and leaves the command successful, so an already-damaged repository is not made harder to inspect.

