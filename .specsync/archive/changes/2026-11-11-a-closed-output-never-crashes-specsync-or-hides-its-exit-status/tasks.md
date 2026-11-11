---
change: a-closed-output-never-crashes-specsync-or-hides-its-exit-status
artifact: tasks
---

# Tasks

- [x] Crate-wide printing macros that drop output after `BrokenPipe` and keep the exit status
- [x] `watch` stops once nobody reads its output
- [x] Integration tests with the reader closed before SpecSync writes; they fail on main
- [x] Error-case rows in the cli and watch specs; CHANGELOG
