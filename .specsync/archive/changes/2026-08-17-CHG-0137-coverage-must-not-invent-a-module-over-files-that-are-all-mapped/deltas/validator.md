## ADDED

### REQUIREMENT REQ-validator-041

Coverage SHALL report a module as lacking a spec only on evidence drawn from that module's own files, never on the absence of a spec directory bearing its name.

Acceptance Criteria
- A candidate whose every discovered file is already mapped by a spec is not reported, in text output and in the machine-readable payload alike.
- A candidate owning at least one unmapped discovered file is still reported.
- A candidate owning no discovered file at all is still reported, because owning nothing measurable is an absence of input rather than evidence of coverage.
- The file and line coverage figures are unaffected: a candidate is suppressed without altering what the report measures.
- Every path from which a candidate name is derived — configured modules, manifest modules, source subdirectories, and flat-file stems — applies the same rule, so a name invented by one derivation cannot survive because another was fixed.
