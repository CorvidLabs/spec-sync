---
change: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
artifact: testing
---

# Testing

A refactor claiming to change nothing has to be proved by counting, because a
diff of 12,543 moved lines is unreadable and a silent loss inside it looks
exactly like a successful move.

    src/change.rs        29,984 -> 17,460 lines   (-42%)
    src/change_tests.rs             12,543 lines

    #[test] functions       309 -> 309
    unit tests passing     2275 -> 2275
    integration passing     374 ->  374
    drill board          42/13 -> 42/13

Every one of those matches main exactly.

`cargo fmt` rewrapped some lines, which is expected: dedenting 12,543 lines by
four spaces lets statements that previously wrapped fit on one line. That was
checked rather than assumed — stripped, non-blank content was compared line by
line before and after formatting, and every difference is a rewrap of the same
tokens. "The formatter changed it" is exactly how a real edit would hide.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-066 (no test altered) | 309 before, 309 after, and the same 2275 unit tests pass. A move that lost a test would still compile and still pass — only the count catches it |
| Private items still reachable | The suite compiles and passes unchanged, which it could not do if `use super::*` had stopped reaching private items; `#[path]` keeps the module inline for exactly this reason |
| No behaviour change | The drill board is 42/13 before and after, byte-identical |
| Helpers correctly retained | 25 `#[cfg(test)]` annotations remain in `change.rs`; production code referencing them still builds |
