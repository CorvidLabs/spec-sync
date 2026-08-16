---
change: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
artifact: design
---

# Design

Move the tests to `src/change_tests.rs` and declare them with

    #[cfg(test)]
    #[path = "change_tests.rs"]
    mod tests;

`#[path]` matters: the module stays INLINE for name resolution, so `use
super::*` still reaches every private item exactly as before. A sibling `mod`
would not, and would have forced visibility changes across hundreds of items —
which is how a "pure move" turns into a real edit.

The extraction turned out to be one contiguous block rather than the scattered
surgery the plan assumed. Of the 25 `#[cfg(test)]` annotations, exactly ONE is
`mod tests`, at line 17457, running to EOF and holding all 309 tests with none
outside it.

The other 24 stay, and it is worth saying why they are not test code: they are
test-only HELPERS and FAULT-INJECTION HOOKS that production paths reference —
`inject_transaction_write_failure`, `record_test_git_process`, test-only
variants of production functions. Moving them would break the production code
that calls them. They live near production code because they instrument it.
