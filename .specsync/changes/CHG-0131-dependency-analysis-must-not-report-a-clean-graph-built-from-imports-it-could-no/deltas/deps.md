## ADDED

### REQUIREMENT REQ-deps-003

Dependency analysis SHALL NOT report a clean graph built from imports it could not read or could not resolve.

Acceptance Criteria
- Kotlin imports resolve against a package topology built first from each JVM file's own `package` declaration and then from directory layout, declaration winning, so a file whose directory does not mirror its package still produces an edge.
- Every imported package resolves to exactly one of: owned by a spec module, foreign to every namespace the project occupies, or inside the project's namespace but unattributed.
- An unattributed import is recorded rather than dropped. When nothing is known about the project's packages, an unowned import is unattributed rather than foreign, so silence is never the default.
- A package claimed by two modules is left unowned and disclosed rather than guessed.
