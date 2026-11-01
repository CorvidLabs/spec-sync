---
change: retain-compound-historical-review-inconsistencies-during-anchored-reopen
artifact: design
---

# Design

Only the existing reopen-only recorded workspace-mismatch mode retains the old review hash as untrusted. Its package is first compared byte-for-byte and mode-for-mode with the remote-default ancestor tree. Definition, execution, closing, implementation tree and finalization hashes remain strict. The old review never gains acceptance authority; closure needs fresh verification and scoped review.
