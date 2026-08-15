## ADDED

### REQUIREMENT REQ-exports-009

Ruby visibility SHALL survive block forms that do not begin a line.

Acceptance Criteria
- A method below `private` is never reported as an export, whether it precedes or follows an assignment-form multi-line conditional.
- Documenting such a method is an orphan error rather than an accepted export, so silencing the warning cannot publish a private method as contract.
- A statement-form conditional, which never desynced the visibility stack, behaves exactly as before.
- Public methods above `private` continue to be extracted.
