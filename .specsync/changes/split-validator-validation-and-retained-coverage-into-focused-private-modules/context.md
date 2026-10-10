---
change: split-validator-validation-and-retained-coverage-into-focused-private-modules
artifact: context
---

# Context

The user requested breaking up SpecSync's large files while preserving behavior and explicitly directed continued work and local branch commits. This independent branch starts at main 815690d and does not depend on the unaccepted MCP branch. Open PR 800 touches change.rs/change_tests.rs and lifecycle documents, not validator paths. Validator has 6416 lines, including about 2470 lines of tests. Its single ordered validation core fits intact below 1000 lines; coverage has coherent retained configuration, capability I/O, traversal, spec inventory and ownership boundaries.
