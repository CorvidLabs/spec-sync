## ADDED

### REQUIREMENT REQ-commands-013

The set of names that cannot be a directory component SHALL have exactly one definition, shared by every part of SpecSync that mints a directory name.

Acceptance Criteria
- The reserved-name check used when validating a module name is the same one used when minting a change's directory name from free text, so the two cannot disagree about whether a name is legal.
- The set is defined once. A second copy is how the two would drift apart, and a name that is reserved for one caller and not the other is a directory some supported platform cannot open.
