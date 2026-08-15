## ADDED

### REQUIREMENT REQ-scoring-003

A spec with a conflicted source file SHALL NOT be awarded API credit.

Acceptance Criteria
- API credit is withheld when any mapped file's extraction unioned both sides of a conflict, because the union describes a tree that does not compile.
- The withholding applies even when the spec's other files parsed cleanly: scoring the readable remainder would report a confident number over an uncompilable tree.
- The reason is explained in the score breakdown rather than presented as a low score with no cause.
