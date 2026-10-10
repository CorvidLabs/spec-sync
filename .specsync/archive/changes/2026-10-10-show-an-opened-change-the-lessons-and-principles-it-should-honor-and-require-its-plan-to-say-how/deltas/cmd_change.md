## ADDED

### REQUIREMENT REQ-cmd-change-018

`change new`, `change status`, and `change show` SHALL render the open-change briefing in text and in JSON. `change finalize` and `change ship` SHALL name the lesson-bundle path in text and in JSON. Their next action SHALL remain the merge instruction, so merging SHALL NOT wait on copying lessons into the spec.

Acceptance Criteria
- The structured result of new, status, and show includes the same briefing the text result prints.
- Finalize and ship text name the bundle path on a line other than the next action.
- The next action of ship, when the change is ready and no sibling changes remain, is still the merge instruction.
- JSON for finalize and ship keeps `lesson_bundle`.

## MODIFIED

### SPEC SECTION Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only: evidence completeness then in-process spec↔code sync. It does not spawn project tests or rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact. Product-stage completion uses the same verification currency predicate, and its action text describes content currency rather than asserting ancestry. The rule covers the scoped review as well as the verification: readiness asks whether the recorded review is current, reports that answer as `current`, `stale`, or `unavailable`, and treats only `current` as satisfied. An unavailable guarantee reported as a satisfied one is worse than the refusal it conceals, and readiness that never asks receives no negative answer and reads its own silence as a pass.
7. Opening, showing, and status render one briefing of the substantive lessons for the change's modules and of the configured principles file, in text and in JSON. A scaffold-only context is omitted. The excerpt is bounded and names the file for the rest. An unreadable file does not fail the command. `change finalize` and `change ship` name the lesson-bundle path in text and in JSON. Their next action stays the merge instruction and does not require copying lessons into the spec. A passing `change check` says nothing about lessons.
8. `status`, `show`, a passing `check`, `approve`, `review`, `finalize`, and ship's finalize each end their text result with exactly one `Handoff:` line — after `Next:` where one is printed — reading `safe`, `conditional`, or `not yet`, an em-dash, the domain's reason, and, when readiness is not safe, `Before clearing:` followed by the domain's steps. The line renders the domain's `HandoffSummary` verbatim: the adapter never decides readiness itself and never prints a digest on it. JSON carries the same object under `summary.handoff` wherever a change summary is rendered and under `handoff` on the approve transition.

### SPEC SECTION Behavioral Examples

### Scenario: Agent creates a change

- **Given** `specsync --json change new "Add passkeys"`
- **When** creation succeeds
- **Then** JSON includes the record, gate summary, and deterministic questions

### Scenario: Agent reopens stale accepted evidence

- **Given** current governed inputs no longer match an accepted change's closing evidence
- **When** `specsync --json change reopen <id> --actor <human> --reason <text>` succeeds
- **Then** JSON contains the verifying change and versioned audit record with the superseded approval and prior verification

### Scenario: Finalize an implementation PR

- **Given** verification and the configured scoped-review check are current
- **When** `specsync change finalize <id>` succeeds
- **Then** output names the dated archive and says the PR is ready for GitHub merge without merging it

### Scenario: Finalize names the lesson bundle

- **Given** a workflow-v2 change is finalized
- **When** `change finalize` or `change ship` prints its result
- **Then** text and JSON name the lesson-bundle path, and the next action is still the merge instruction

