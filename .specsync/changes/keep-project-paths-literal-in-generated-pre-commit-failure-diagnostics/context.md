---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: context
---

# Context

This is a bounded repair of reported security issue #782, reproduced against released SpecSync 6.1.1. It preserves the hook API, scoped project identity, repository guard, enforcement and idempotent managed-block behavior. The proposed definition remains unapproved; implementation review, lifecycle finalization and release remain separate gates.
