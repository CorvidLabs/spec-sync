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

### REQ-cmd-change-006

The change command adapter SHALL scope shared lifecycle reads to one command invocation without
altering output, exit, or mutation behavior.

Acceptance Criteria

- List, show, status, and check create and drop one domain read snapshot around their established
  operation.
- Text and JSON projections remain semantically identical to uncached domain results.
- No approve, start, verify, accept, reopen, correction, creation, archival, or adoption command
  installs a read snapshot.
