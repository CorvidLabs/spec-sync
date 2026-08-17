## ADDED

### REQUIREMENT REQ-cmd-lifecycle-003

The lifecycle minimum-score gate SHALL remain inclusive and SHALL refuse a directory mapping on the same basis as `check`.

Acceptance Criteria
- A total equal to the configured minimum passes; only a total below it fails.
- A spec whose `files:` entry is a directory scores zero and therefore fails any positive minimum, matching the hard failure `check` already produces for the same mapping.
