---
change: CHG-0046-make-lifecycle-verification-workflows-evaluate-the-exact-pull-request-head-while
artifact: docs
---

# Docs

This is an internal workflow correction. Public product and migration documentation do not
change. The governed record documents that exact-head checkout is required only where a job
compares persisted lifecycle evidence with Git ancestry; synthetic-merge checkout remains the
correct default for ordinary pull-request integration testing.
