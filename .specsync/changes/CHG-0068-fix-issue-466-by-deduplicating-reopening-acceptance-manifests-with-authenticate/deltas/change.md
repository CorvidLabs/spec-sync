## ADDED

### REQUIREMENT REQ-change-043

Audited reopening history SHALL store each distinct prior acceptance manifest once as an immutable
content-addressed object and append only a bounded authenticated reference in each new reopening
event, while preserving fail-closed access to legacy embedded evidence.

Acceptance Criteria

- Repeated reopen cycles never duplicate an identical full acceptance manifest in new approval
  ledger events.
- A compact reference resolves only through a validated digest-derived path beneath the exact
  active or archived change workspace.
- Missing, malformed, path-unsafe, symlinked, digest-mismatched, or semantically inconsistent
  manifest objects fail closed before reopening history is trusted.
- Existing schema-v1 embedded reopening records remain readable and verifiable without bulk
  rewriting, while new events use the compact versioned representation.
- A large-manifest A/B/A reopen history stores exactly two distinct immutable objects and grows the
  approval ledger only by bounded event metadata.
- Reopen, verify, accept, archive, and `migrate 5.0` preserve authenticated deterministic history
  across legacy and compact representations.
