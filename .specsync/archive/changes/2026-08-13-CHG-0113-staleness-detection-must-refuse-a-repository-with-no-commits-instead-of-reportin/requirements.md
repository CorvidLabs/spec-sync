---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: requirements
---

# Requirements

## REQ-git-utils-002

Git helpers SHALL expose whether a repository has any history, distinctly from whether a
path is a work tree.

Acceptance Criteria
- A repository with at least one commit reports that it has history.
- A repository with an unborn HEAD reports that it does not, while still reporting as a work tree.
- A path that is not a repository reports neither.

## REQ-cmd-stale-002

Staleness detection SHALL refuse to report a verdict when there is no history to derive one
from, and SHALL name which precondition is missing.

Acceptance Criteria
- A repository with no commits is refused and exits non-zero, rather than reporting specs up to date.
- A path that is not a repository continues to be refused and exits non-zero.
- The two causes are reported distinctly in both text and machine-readable output.
- A repository with at least one commit reports staleness normally and exits zero.
