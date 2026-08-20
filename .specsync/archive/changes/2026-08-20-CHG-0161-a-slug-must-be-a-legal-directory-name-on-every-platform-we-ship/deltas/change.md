## ADDED

### REQUIREMENT REQ-change-083

A minted change slug SHALL be a legal directory component on every platform SpecSync ships a binary for, and SHALL remain readable when the description is too long to keep.

Acceptance Criteria
- The length limit bounds the bytes of the name that reaches the filesystem rather than the characters of the description it came from, and is sized so the deepest path a change produces stays within the shortest maximum path length of any supported platform.
- A name that must be shortened is cut at a word boundary rather than mid-word whenever a boundary is near enough for the result to stay legible, because the description is stored in full elsewhere and the directory name exists to be read.
- A description that would reduce to a reserved directory name does not become one, including the name substituted when a description reduces to nothing.
- A description that needs none of this produces exactly the name it produced before.
