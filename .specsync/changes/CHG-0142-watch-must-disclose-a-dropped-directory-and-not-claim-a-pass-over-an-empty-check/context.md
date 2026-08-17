---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: context
---

# Context

`specsync watch` builds its watch set by testing each configured directory with
`is_dir()` and keeping the ones that pass. A configured directory that does not
exist — a typo, a path that moved, a `specs_dir` that was never generated — fell
out of the set with no record that it had ever been named. The banner then
printed the directories that survived, which reads as a complete list of what is
being monitored. Nothing said otherwise.

That is the release's defect class one more time: the watch set is short for want
of INPUT, and every consumer downstream reads it as the full set. The banner is
not merely incomplete, it is wrong — it answers "what am I watching?" with a
list that omits exactly the path the operator got wrong.

The second half is the same mistake one level down. `watch` forks `specsync
check` and reads the child's exit status. A check that finds no specs prints
`No spec files found in <dir>/` and exits 0, because in bare mode that is
informational, not a failure (#560 settled that `--strict` is where it gates).
`watch` had no way to tell "examined N specs, all clean" from "examined none",
so it printed a green `All checks passed!` over a run that checked nothing. An
operator who mistyped `specs_dir` saw a dropped directory they were not told
about and a pass they did not earn.

Both halves are visible together in the sandbox: drill 060 gates on the
disclosure AND on the false all-clear, and it was the false all-clear that
survived the first fix.

## Why the check command is untouched

`check --strict` already fails this tree — #560 fixed that and drill 060 keeps a
control on it. The bug is that `watch` narrates a check it did not evaluate,
not that the check is wrong.
