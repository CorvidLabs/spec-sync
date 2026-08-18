# Context

`change ship` of a change with no semantic deltas copies an empty `deltas/` into the dated
archive package. Git cannot track an empty directory, so the commit records the package's
files but not that directory. Checking out any commit that predates the archive removes every
tracked file and leaves the dated directory behind holding nothing but `deltas/`.

`git status --short` prints nothing — there is no untracked *file* to report — while
`change new`, `change audit`, `change adopt` and `check` all read
`<dated dir>/state.json`, find ENOENT, and surface the raw OS error. Measured on
34ade838:

    change new        error: failed to read archived change state …/state.json:
                      No such file or directory (os error 2)          rc=1
    change audit      same                                            rc=1
    change adopt      same                                            rc=1
    check             same, degraded to a warning                     rc=0
    check --strict    same                                            rc=1

No verb names the remedy (`rm -rf` the dated directory).

## Why this is the release's defect class

The directory is empty *for want of input* — the change had no semantic deltas — and the code
reads that emptiness as a *corrupt archive*. The active-side reader already carries the exact
opposite reading, added deliberately with a comment at `src/change.rs:1892`:

    // Git cannot track empty directories, so checking out a branch without this
    // change leaves a husk behind — typically just an empty `deltas/`. […]
    // A directory with no `state.json` is not an active change here.

That tolerance was applied to the active arm of `located_change_sequences` and to
`list_changes_checked`, and not to the archive arm of the same function nor to
`list_all_changes_uncached`. This is the sibling-site pattern: the fix landed where the report
pointed and the parallel implementations survived.
