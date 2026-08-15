---
change: CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused
artifact: docs
---

# Docs

CHANGELOG under Unreleased → Fixed, leading with the consequence: `check`
reported `3/3 exports documented` and exit 0 for a file that does not compile,
because the symbol list was the union of two alternative trees.
