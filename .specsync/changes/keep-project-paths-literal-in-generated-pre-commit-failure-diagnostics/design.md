---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: design
---

# Design

Build the scoped checker command once, replace only its exact executable template line, and independently shell-quote the entire failure hint as one argument to fixed-format printf. Preserve project marker ownership, user content, worktree/core.hooksPath support and all current success/failure semantics. No dependency, executable lookup policy or public API changes.
