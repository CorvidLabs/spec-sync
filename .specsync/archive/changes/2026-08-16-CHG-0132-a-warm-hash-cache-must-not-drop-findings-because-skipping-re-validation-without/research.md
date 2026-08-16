---
change: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
artifact: research
---

# Research

The distinguishing question — never-stored versus stored-and-not-replayed — was
asked before implementation because the two have different fixes and the same
symptom. The answer was never-stored, with the storage types already present.

This is the second time in this campaign that a bug turned out to be unwired
machinery rather than absent machinery. The first was #578, whose detector was
complete and short-circuited by `if true { return None; }`. Both cases share a
property worth watching for: a reader auditing the codebase would conclude the
capability exists.
