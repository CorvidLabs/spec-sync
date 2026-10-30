## ADDED

### REQUIREMENT REQ-generator-008

Source selection SHALL accept confined exact regular source files alongside directory entries without widening their scope.

Acceptance Criteria
- Preserve explicit mapping precedence, configured extensions, test and exclusion filters, no-follow root confinement, deterministic deduplication and exactly-one-file fallback; overlapping entries are not ambiguous and distinct candidates remain ambiguous.
