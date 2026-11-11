---
change: the-recorded-base-order-test-uses-full-commit-ids
artifact: context
---

# Context

CI on #1052's archive tip f3605f2c failed one unit test whose fixture used short fake base ids, which the
full-commit-id rule now ignores. This change covers the fixture update.
