---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: tasks
---

# Tasks

- [x] Implement the versioned correction ledger and effective metadata/artifact projection.
- [x] Implement portable correction-chain and complete-definition digest validation.
- [x] Implement atomic `accepted` to `verifying` correction without canonical replay.
- [x] Require fresh definition, verification, and closing gates after correction.
- [x] Add `change correct` grammar plus deterministic text and JSON rendering.
- [x] Surface correction history and gate health through show and status.
- [x] Add unit and integration coverage for every correction and failure path.
- [x] Update canonical companions, workflow/CLI docs, and the unreleased changelog.
- [x] Pass every pre-acceptance trust step; rerun strict spec validation after the approved Public API delta becomes canonical.
