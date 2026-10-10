## ADDED

### REQUIREMENT REQ-cmd-init-007
Initialization SHALL keep a `*.3md` file from being classified as Roff.

Acceptance Criteria
- A project with no `.gitattributes` receives `*.3md linguist-language=Text`.
- A project that already sets a Linguist language for `*.3md` is left unchanged.
- Running initialization again does not add a second copy of the rule.
