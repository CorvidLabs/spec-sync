---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: context
---

# Context

This is a bounded repair of reported security issue #782, reproduced against released SpecSync 6.1.1. It preserves the hook API, scoped project identity, repository guard, enforcement and idempotent managed-block behavior. Leif directly approved this prepared definition on 2026-10-16, conditional on actual local testing and at least 95% engineering confidence. The native writer recorded the genuine scope approval as actor 0xLeif; the direct response is preserved in the coordinator receipt ci-cleanup-human-approval.json. Independent implementation review, current qualification, lifecycle finalization and release remain separate gates.
