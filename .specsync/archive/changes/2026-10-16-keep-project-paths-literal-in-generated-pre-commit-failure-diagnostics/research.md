---
change: keep-project-paths-literal-in-generated-pre-commit-failure-diagnostics
artifact: research
---

# Research

The global replacement of `specsync check` inserted the shell-quoted command into both executable code and a double-quoted echo diagnostic. Quoting appropriate for the command argument was unsafe inside that diagnostic. Released 6.1.1 executes harmless dollar/backtick marker substitutions on failed checks and breaks an apostrophe path. The five executable regressions fail before the patch (2 pass, 3 fail) and the complete hooks test selection passes afterward (63 tests). Existing installation returns early on matching markers, so upgrade guidance must require exact-project uninstall/reinstall; automatic refresh is outside this change.
