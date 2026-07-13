---
change: CHG-0021-close-reopened-lifecycle-review-gaps
artifact: context
---

# Context

The audited reopen path correctly prevents canonical replay and preserves prior
evidence, but four boundary cases could disagree: strict check could accept a
definition that closing rejects, reapproval could leave the failing lane,
nested projects could miss repository history, and a non-stale delivery could
reopen because of an unrelated closing-validity error.
