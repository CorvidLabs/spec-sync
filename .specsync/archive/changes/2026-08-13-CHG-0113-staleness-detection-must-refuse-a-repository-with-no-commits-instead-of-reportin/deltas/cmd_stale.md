## ADDED

### REQUIREMENT REQ-cmd-stale-002

Staleness detection SHALL refuse to report a verdict when there is no history to derive one
from, and SHALL name which precondition is missing.

Acceptance Criteria
- A repository with no commits is refused and exits non-zero, rather than reporting specs up to date.
- A path that is not a repository continues to be refused and exits non-zero.
- The two causes are reported distinctly in both text and machine-readable output.
- A repository with at least one commit reports staleness normally and exits zero.
