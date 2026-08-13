## ADDED

### REQUIREMENT REQ-deps-002

Dependency analysis SHALL report every spec it dropped, and SHALL NOT declare a graph valid
when any declaration could not be read.

Acceptance Criteria
- Frontmatter errors surfaced by the parser are reported rather than discarded, using the same wording the validator emits.
- A spec whose frontmatter cannot be parsed, and a spec declaring no module, are each reported as excluded from the analysis.
- Any such report is an error, so the command exits non-zero and does not print that all declarations are valid.
- A project whose specs are all well-formed continues to report a valid graph and exit zero.
