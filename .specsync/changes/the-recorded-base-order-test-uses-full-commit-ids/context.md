---
change: the-recorded-base-order-test-uses-full-commit-ids
artifact: context
---

# Context

CI on #1052's archive tip f3605f2c failed one unit test whose fixture used short fake base ids, which the
full-commit-id rule ignores. corvid-agent's review of #1052 also asked that only the oldest change's base
count, as the acceptance criterion says, and that the status evidence share the commit-id helper.
