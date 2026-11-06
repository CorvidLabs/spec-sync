---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: design
---

# Design

**A. One walk for the archive-introduction index.** Replace the per-commit `diff-tree` with one `git log --format=%x00%H --diff-filter=A --no-renames --name-only -z -m --root <refs> -- <archive root>`. Group the added paths by commit, and keep only `<archive root>/<package>/state.json`. `-m` lists a merge commit's additions against every parent, as `diff-tree -m` did. The same `--max-count` bound applies to commits. The `state.json` and `approvals.json` reads go through `run_git_bounded` with full object ids, which the batch reader from #861 already answers. The per-commit entries stay in the persistent cache, so a partly warm cache still helps.

**B. The cutoff tree decides the workflow-v1 check.** "The record existed at the cutoff" asks only about paths in the cutoff commit's tree, and the baseline history check has already proven that commit reachable. So one `ls-tree -r --name-only -z <cutoff> -- <changes root> <archive root>` per command, memoized, lists every candidate `state.json` at the cutoff. Each workflow-v1 change then checks its active path and any `<archive root>/*-<id>/state.json` in that list, reading the record at the cutoff as today. The yes/no verdict per `(cutoff, id)` goes into the persistent read cache. The cutoff is an immutable commit, so the entry never needs invalidating. A rewritten baseline names a different cutoff, which is a different key.

**Deadline.** A thread-local read deadline, set by the command layer from `--timeout` or `SPECSYNC_TIMEOUT`. The long loops (archive-introduction index, per-change validation in `list_all_changes_loaded`, workflow history) call `read_deadline::check(phase, done, total)` between items. When the deadline has passed, that returns `Err(ReadTimedOut { phase, progress })`, which is carried up as a typed error. The persistent read cache frame flushes what it holds on that error, as well as on success. The command layer turns it into exit 124 and the text or JSON report, with the resume command built from the command's own argv minus `--timeout`. Lifecycle writes never take a deadline.
