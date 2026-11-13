---
change: a-late-close-refuses-a-package-holding-a-file-git-ignores
artifact: context
---

# Context

Issue #1008 item 2, a Codex P2 on #994 tracked under Leif's two-round cap. `plan_late_close`
(`src/change/doctor.rs`) checks that the package equals the remote default with
`git diff --quiet <default> -- <package>` and `git ls-files --others --exclude-standard`. With
`--exclude-standard`, a file Git ignores inside the package never blocks the close, though it was
never committed, and the move then carries it into the closed-late package. REQ-change-143 moves a
package "committed on the remote default branch ... byte for byte". Dropping the flag counts every
untracked file in the package, ignored ones included. No requirement text changes.
