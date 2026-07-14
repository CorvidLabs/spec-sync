---
change: CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re
artifact: tasks
---

# Tasks

- [ ] Implement the versioned correction ledger and effective metadata/artifact projection.
- [ ] Implement portable correction-chain and complete-definition digest validation.
- [ ] Implement atomic `accepted` to `verifying` correction without canonical replay.
- [ ] Require fresh definition, verification, and closing gates after correction.
- [ ] Add `change correct` grammar plus deterministic text and JSON rendering.
- [ ] Surface correction history and gate health through show and status.
- [ ] Add unit and integration coverage for every correction and failure path.
- [ ] Update canonical companions, workflow/CLI docs, and the unreleased changelog.
- [ ] Pass full local verification and `fledge trust verify`.
