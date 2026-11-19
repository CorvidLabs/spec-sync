---
change: fix-the-four-pr-1086-review-findings-on-history-fragments
artifact: context
---

# Context

Follow-up to #1084 / PR #1086. Codex left four findings on #1086. corvid-agent's approval judged none blocking, and Leif chose "Merge, fix all 4 in a follow-up before release", relayed by hc t4c on 2026-11-18. #1086 merged as 3bebe8b0. Its four threads were answered with a link to this PR (#1088) and resolved.

The tests and fixes were written before this change was opened, so #1088 could be linked from #1086's threads before #1086 merged. Each test failed on 979c3fe9, the #1086 tip. The change was then opened on the branch rebased onto 3bebe8b0.
