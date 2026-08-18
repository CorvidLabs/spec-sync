## ADDED

### REQUIREMENT REQ-git-utils-004

A shared predicate SHALL answer whether a cited path was known to git at a given commit and is now absent.

Acceptance Criteria
- The predicate distinguishes a deletion, which git can state and name a commit for, from a path git never tracked, whose drift is genuinely unknown.
- Paths are resolved relative to the project root, so a project inside a subdirectory of the repository is answered correctly.
- Every command that answers a staleness question consumes this predicate rather than re-deriving the distinction, so the answers cannot diverge.
