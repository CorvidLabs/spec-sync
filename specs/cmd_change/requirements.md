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

`specsync change ship-status` SHALL report local ship readiness for an active change
without querying GitHub check-runs: verification tip presence and ancestry relative
to HEAD, whether a scoped review is recorded, blockers, warnings (including
merge-before-finalize), and a concrete next action.

Acceptance Criteria

- JSON includes `verification_present`, `verification_ancestor_of_head`,
  `review_present`, `ready_to_finalize`, `blockers`, `warnings`, and `ship_next`.
- An absent or non-ancestor verification commit is a blocker naming re-check.
- A verifying change always warns not to merge before finalize.

