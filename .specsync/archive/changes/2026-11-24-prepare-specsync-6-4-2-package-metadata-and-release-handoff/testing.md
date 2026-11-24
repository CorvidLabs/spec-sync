---
change: prepare-specsync-6-4-2-package-metadata-and-release-handoff
artifact: testing
---

# Testing

Run the distribution version validator, workflow runtime pin validator and release-candidate contract tests. Check both the candidate inputs and the published default in an isolated checkout. Confirm the release notes describe the diff from v6.4.1 to the selected source.

Run the full verification lane, strict check with forced 100 percent coverage, scoped verification, strict change audit, the pre-push gate and configured Trust. Record the exact commit for each result. The candidate workflow supplies the Linux and macOS qualification evidence after tagging.
