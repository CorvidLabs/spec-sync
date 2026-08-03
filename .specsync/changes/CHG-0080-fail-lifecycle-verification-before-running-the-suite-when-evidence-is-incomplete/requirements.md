---
change: CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete
artifact: requirements
---

# Requirements

## REQ-change-049: Verification fails fast, explains itself, and converges

Evidence completeness is derived from committed artifacts alone, so it is resolved before any
verification command runs. Messages name the artifact to edit and the command that failed.
Applying a delta whose effect is already present converges instead of erroring, and duplicate
change ordinals from one base commit are rejected.
