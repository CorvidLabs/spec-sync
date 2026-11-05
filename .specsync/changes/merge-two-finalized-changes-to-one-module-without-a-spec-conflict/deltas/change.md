## ADDED

### REQUIREMENT REQ-change-125

`change audit` SHALL refuse a living requirements file that contains the same requirement ID more than once, and the error SHALL name that ID.

Acceptance Criteria
- Two `### REQ-<module>-<number>` headings with the same ID in one `requirements.md` fail `specsync change audit`.
- The error text contains that ID.
- The refusal is reported even when SDD policy is disabled.
- A fenced copy of the heading is not a second requirement.
- No other audit refusal is weakened.
