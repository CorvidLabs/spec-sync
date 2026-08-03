## ADDED

### REQUIREMENT REQ-change-049

Lifecycle verification SHALL resolve evidence completeness before running any
verification command, SHALL name the artifact and section an author must edit to close an
evidence gap, and SHALL name the failing command when a command fails. Delta application
SHALL converge when an `## ADDED` block is already present with byte-identical content, and
SHALL reject a duplicate `CHG-NNNN` ordinal claimed by two distinct changes from the same
base commit.

Acceptance Criteria

- Incomplete acceptance or requirement evidence fails before any verification command runs.
- The evidence-gap message names the change `testing.md` and its `## Requirement evidence`
  table; the command-failure message names the failing command and its exit code.
- An `## ADDED` block already present with byte-identical content applies as a no-op, so
  re-deriving the canonical tree converges.
- An `## ADDED` block present with different content fails and directs the author to
  `## MODIFIED`.
- Two distinct changes claiming one ordinal from the same base commit are rejected at
  definition approval and by `change audit`; differing or unknown base commits are accepted.
