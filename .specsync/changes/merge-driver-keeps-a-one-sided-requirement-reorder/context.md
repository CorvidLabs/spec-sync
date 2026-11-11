---
change: merge-driver-keeps-a-one-sided-requirement-reorder
artifact: context
---

# Context

Issue #874 (follow-up from #865's review): `merge_requirements` seeds the
output order from the ancestor's ID sequence, so when one branch reorders
existing requirement blocks and the other only adds or edits one, the merge
reports clean and silently drops the reorder. Fix: compare each side's
surviving-ID sequence against the ancestor's; keep a one-sided reorder; when
both sides reordered differently, mark the merge conflicted rather than
picking one silently.
