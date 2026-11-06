---
change: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
artifact: tasks
---

# Tasks

- [x] Run the three self-exec'd scripts as `sh <path>` (git_utils x2, substantive_plan x1).
- [x] Write the three fake `gh` executables from a child `sh` process (github.rs x2, integration issues x1).
- [x] Re-run the 300-round Linux stress runs with the change and record the results.
