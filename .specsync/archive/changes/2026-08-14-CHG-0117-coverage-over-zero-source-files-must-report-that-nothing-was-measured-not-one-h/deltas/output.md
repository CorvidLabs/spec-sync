## ADDED

### REQUIREMENT REQ-output-004

Coverage output SHALL NOT report a percentage when there was nothing to measure, and SHALL
NOT make affirmative claims that are true only of an empty set.

Acceptance Criteria
- A zero file denominator reports that there were no source files to measure, rather than a percentage.
- A zero line denominator reports that there were no source lines to measure, rather than a percentage.
- When no source files were found, the claims that every source file is referenced and every module has a spec directory are not printed.
- In their place the likely cause is named, so a misconfigured source directory or an over-broad exclusion can be corrected.
- A project containing source files reports its percentages and affirmative lines unchanged.
