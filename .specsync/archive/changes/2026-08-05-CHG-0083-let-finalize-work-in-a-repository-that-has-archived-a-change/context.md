---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: context
---

# Context

Six pull requests in the 6.0 cycle were merged at `check`, skipping `review` and
`finalize`. Each stranded its change in `verifying`, where it stayed in the
active set and was staled by every later commit. main went red; each branch cut
from it inherited the failure.

That looked like process error. It was not: `finalize` could not succeed in this
repository at all, because it has archived changes. Merging at `check` was the
only way forward. The workaround was rational; the defect was upstream of it.

Lesson: every fixture in both suites — 2,181 Rust tests and every sandbox drill —
builds from an empty temp repository. Defects that require accumulated history
cannot fail there by construction. That blind spot hid a total failure of the
lifecycle terminal step across an entire release cycle.
