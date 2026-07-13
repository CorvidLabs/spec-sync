## MODIFIED

### REQUIREMENT REQ-change-020
Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, semantic successor governance, and validation of every current canonical contract it reapproves.

Acceptance Criteria
- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy the canonical-successor fallback, even when its affected paths and specs overlap.
- A later recorded semantic canonical change can satisfy successor governance for every overlapping affected spec and path.
- A reopened canonical-applied change validates its current canonical modules without replaying its already-applied semantic delta.
- Strict project checks reject a reopened definition that reacceptance would reject.
- Definition reapproval keeps a canonical-applied reopened record in the verifying state so fresh evidence remains mandatory.
- Nested project history lookup anchors repository-relative workspace state paths at the Git repository top.
- Reopen rejects a request when current delivery inputs match accepted evidence, regardless of another closing-validity failure.
