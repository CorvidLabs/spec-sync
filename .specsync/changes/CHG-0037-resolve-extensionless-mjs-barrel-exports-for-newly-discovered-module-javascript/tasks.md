---
change: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
artifact: tasks
---

# Tasks

- [x] Reproduce the missing extensionless `.mjs` barrel export against the current PR head.
- [x] Identify the shared one-level relative resolver used by both parsing modes.
- [x] Define focused sibling, directory-index, and strict-validation regression fixtures.
- [x] Preserve CHG-0036 and declare the supported lifecycle dependency instead of rewriting accepted history.
