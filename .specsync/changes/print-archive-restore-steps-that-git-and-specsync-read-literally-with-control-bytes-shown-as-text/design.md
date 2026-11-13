---
change: print-archive-restore-steps-that-git-and-specsync-read-literally-with-control-bytes-shown-as-text
artifact: design
---

# Design

- Pathspecs: the printed restore and archiving-commit lookup, and both probes behind them
  (`archive_package_differs_from` and the `log --diff-filter=A` lookup), pass the package as
  `:(literal)<package>` (relative to the project root, which is where the step runs). The shell
  word is still one quoted word.
- Change id: the printed status commands become `specsync change status -- '<id>'` and
  `specsync change status --json -- '<id>'`, with options before `--`.
- Control bytes: a new `shell_word` quotes a word that holds no control byte exactly as
  `shell_quote` does today (single quotes; `'` as `'\\''`). A word that holds one is written as
  `$'…'`, with `\\\\`, `\\'` and `\\xHH` for each control byte (C0 and DEL). So the printed
  step holds no control byte, and bash still reads the exact name. Only `corrupt_archive_recovery`
  switches to it; other callers of `shell_quote` are untouched.
- Nothing about the verdicts changes: the same three cases, in the same order, chosen by the
  same probes.
