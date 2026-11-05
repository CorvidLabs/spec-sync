---
change: finalize-validates-only-the-change-it-archives
artifact: requirements
---

# Requirements

REQ-change-124 (delta `deltas/change.md`) restates the acceptance criteria of #856:

- The Git processes one finalize spawns do not grow with the number of other archived changes.
- Every refusal finalize issues for the change being archived is unchanged.
- A hidden downgrade in another archive is still caught by `change audit --archives`.
- A Raven-sized finalize finishes in seconds.
