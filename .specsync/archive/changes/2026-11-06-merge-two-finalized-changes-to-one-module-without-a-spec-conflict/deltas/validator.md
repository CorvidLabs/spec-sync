## ADDED

### REQUIREMENT REQ-validator-045

`check` SHALL refuse a requirements file that contains the same requirement ID more than once, and the error SHALL name that ID.

Acceptance Criteria
- Two `### REQ-<module>-<number>` headings with the same ID in one `requirements.md` fail `specsync check`.
- The error text contains that ID and the requirements path.
- The same ID written inside a fenced code block is not a second requirement.
- A file with each ID once still passes this check.
