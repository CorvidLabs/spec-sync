## MODIFIED

### REQUIREMENT REQ-cmd-change-001

The system SHALL provide equivalent text and JSON interfaces for the complete SDD lifecycle and its explicit semantic-succession adoption.

Acceptance Criteria
- Humans receive concise next-action guidance and one consistent accepted-validity reason.
- Agents receive stable records, summaries, question identifiers, manifests, supersedes edges, and successor evidence.
- `change supersede` records explicit predecessor/path/module/digest obligations only before definition approval.
- `change approve --portable-5-0-1` records one atomic marked pair and renders it as one definition transition.
- Audited reopen returns the verifying change and its versioned supersession event in deterministic JSON.
- Active accepted check/status/reopen/archive eligibility render exact, successor-covered, provisionally-covered, or stale; archived status renders authenticated-history or corrupt-history.
