## ADDED

### REQUIREMENT REQ-cmd-check-003

The primary check command SHALL consume one fallible schema snapshot and report warning
suppression truthfully in every supported output.

Acceptance Criteria

- Schema replay, table identity, pattern additions, and column validation derive from one immutable
  snapshot per validation invocation.
- Missing, unreadable, malformed, or vacuous configured schema input is an explicit finding and
  cannot become an empty successful comparison.
- Text, JSON, Markdown, and GitHub output distinguish emitted warnings from deterministic
  `suppressed_warnings` details.
- Strict exit behavior counts unsuppressed findings only and preserves existing cache and coverage
  semantics.
