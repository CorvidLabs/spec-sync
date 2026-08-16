---
change: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
artifact: context
---

# Context

`src/change.rs` was 29,983 lines — 23% of the codebase in one file, holding 829
functions and 309 tests.

That size is not cosmetic, and this change exists because of what it causes. The
defect this release has spent its correctness campaign on is a fix landing at
the site named in the bug report while a parallel implementation survives. It
has now happened seven times: #558 fixed `stale.rs` and left three other
staleness readers; #562 fixed `output.rs` and left eight coverage sites; #570
guarded one config loader and left the second; #474's wrapper had five callers,
four unreported; #477's own fix reproduced the defect one layer down. You cannot
sweep for a sibling in a file you cannot hold in your head.

Ten of the thirteen remaining red drills are lifecycle bugs, and every one of
them lives in this file. Fixing them first, in a 30,000-line file, is doing the
hard work in exactly the condition that produced the bugs.
