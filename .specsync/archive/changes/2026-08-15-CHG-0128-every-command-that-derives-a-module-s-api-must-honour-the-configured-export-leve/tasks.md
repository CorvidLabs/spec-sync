---
change: CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve
artifact: tasks
---

# Tasks

1. Thread `&SpecSyncConfig` to every derive site; take level and mode from it.
2. Retain the wrappers with `#[allow(dead_code)]` and a warning doc comment.
3. Unit tests asserting the configured surface is used, that fail on unfixed code.
4. CHANGELOG entry.
