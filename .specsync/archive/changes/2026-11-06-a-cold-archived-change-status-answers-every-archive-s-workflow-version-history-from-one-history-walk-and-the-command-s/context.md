---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: context
---

# Context

Issue #915, the follow-up S9 (#918) named. #918 brought a cold `change status <archived-id>` from 14,482 Git processes to about 5,600 on this repository, but it still grows with the archive count: 34.1 s and 3,899 processes on the Raven-sized benchmark, and 99 then 234 processes in the guard fixture for 2 then 6 archives. The cold under-5 s budget from S9's approval was measured and reported there, not required.

Leif chose option 3 of the #915 proposal and approved SPEED-1.a.1 as drafted on the morning of 2026-11-06 ("Ok yes I approve all"), relayed by orc t18. Option 3 is one history walk for all archives plus the command's single batch reader, with every archive still fully validated. Option 2 (validate only the target archive) was not chosen, so `authenticated-history` keeps its meaning. The orc reserved REQ-change-140 and REQ-change-141 for this change.

This branch is stacked on #918 at `1737406b`. The failing guard is `0623412f`. When #918 merges, origin/main is merged in here before the PR.

The unscoped read (#921, `change correct`) is not in scope. It opens no read scope, so it gets none of this.
