---
change: keep-archived-coverage-recoverable-and-visible
artifact: context
---

# Context

Two isolated command-line reproductions on the #1132 integration at `1abf8b80` show that a successful archived reopen strands the previous generation’s valid `coverage.json` in the active package, and that three read commands silently treat corrupt coverage as absent. Git history still authenticates the archive in both cases. The proposed repair is a distinct scope from #1132’s explicit succession work.
