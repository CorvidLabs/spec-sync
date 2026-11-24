---
change: close-fenced-delta-replay-review-findings
artifact: context
---

# Context

The review of PR #1144 found that canonical-block matching called the general spec parser's fence blanker, while the requirement reader has its own CommonMark fence recognizer. A backtick in a proposed opener's info string therefore hides a real heading from materialization while the requirement reader sees it. The review also asks for regression coverage of the shared matcher's already-applied removal and modification paths.

This follow-up adds three tests and uses the existing validator helper. It keeps the original archive and review evidence on the PR.
