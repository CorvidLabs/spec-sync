---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: testing
---

# Testing

Four fixtures, built fresh for every measurement and proven byte-identical apart
from git state:

    f-stale         git, 7 commits, spec 6 behind its only source  (drifted control)
    f-stale-nogit   same tree, .git removed
    f-stale-unborn  same tree, `git init` only
    f-fresh         git, 1 commit, in sync                          (clean control)

`diff -r --exclude=.git` reports the three drifted trees identical, so git state
is the only variable.

The matrix is four fixtures by nine commands: stale, stale --json, report,
report --json, check --stale, check --stale --json, check, coverage, score
--json. Before and after are diffed whole.

**Exactly six cells change**: report{text,json} and check --stale{text,json} on
the two unmeasurable fixtures. Every other cell is byte-identical, including all
nine commands on both healthy fixtures and `stale`, plain `check`, `coverage`
and `score` on the broken ones. That diff is the evidence for both directions at
once — a change that refused too broadly would show far more than six.

Discrimination: with `missing_history` forced to always return `None` — the
pre-fix behaviour at every site simultaneously — the staleness integration tests
fail 8 of 13, and the survivors are exactly the healthy-fixture controls, which
is what a correct control set should do.

Suite: fmt clean, clippy clean, 2225 unit + 355 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-git-utils-003 | `MissingHistory` distinguishes the two states; `stale`'s output is unchanged after refactoring onto it, verified by transcript diff — which is what proves the strings were preserved rather than re-invented |
| REQ-cmd-report-003 | Both unmeasurable fixtures exit 1 with the reason named; JSON carries `stale_modules: null`. The guard sits after the coverage computation, so the pre-existing inconclusive-coverage contracts still hold |
| REQ-cmd-check-010 | `check --stale` exits 1 on both; `"stale"` is `null`, not `[]`. Plain `check` on the same fixtures is byte-identical to before, which is the cell that proves the guard is scoped to the flag |
| REQ-cmd-stale-003 | Messages, JSON and exit codes byte-identical; the duplicated precondition is gone, so #558's fix now reaches every reader instead of one |
| REQ-cmd-lifecycle-002 | The `no_stale` guard reports `staleness unverifiable — <reason>` instead of passing |
| REQ-scoring-002 | Two trees differing only by `.git`: freshness no longer scores 20/20 on the tree git cannot read, so removing the repository can no longer raise the grade from C to B |
| REQ-mcp-005 | The MCP staleness surface reports the unmeasurable state rather than zero drift |
