---
spec: cmd_change.spec.md
---

# Requirements

### REQ-cmd-change-001

The system SHALL provide equivalent text and JSON interfaces for the complete SDD lifecycle.

#### Acceptance Criteria

- Humans receive concise next-action guidance.
- Agents receive stable records, summaries, and question identifiers.
- Audited reopen returns the verifying change and its versioned supersession event in deterministic JSON.

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

