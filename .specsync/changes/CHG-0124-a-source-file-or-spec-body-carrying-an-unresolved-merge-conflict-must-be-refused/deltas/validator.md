## ADDED

### REQUIREMENT REQ-validator-015

A spec whose mapped source is conflicted, or whose own body carries a conflict, SHALL fail validation.

Acceptance Criteria
- The spec is not compared against a union of two alternative trees.
- A body conflict is reported before frontmatter parsing, so it is named as a conflict rather than as an incidental parse error.
- A path git reports as unmerged is refused whatever the extractor made of its bytes.
- Every read path is covered, including the pre-read snapshot path used by `issues`.
