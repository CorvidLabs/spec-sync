---
change: CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check
artifact: design
---

# Design

## Resolving the watch set

The watch set was built by filtering: test each configured directory with `is_dir()`,
keep the passers. A filter discards the reason it discarded — the caller receives a
shorter list and cannot tell a complete one from a truncated one.

`resolve_watch_dirs` replaces the filter with a partition:

```rust
struct WatchDirs {
    watched: Vec<PathBuf>,
    skipped: Vec<(String, String)>,   // (configured path as written, role)
}
```

The skipped half carries the path **as written in the config**, not the absolute path it
resolved to, because the operator has to find the typo in their own file. It carries the
role (`specs_dir` or `source_dirs`) because a project with several source directories
otherwise learns only that something is wrong, not which setting.

The disclosure goes to **stderr** in both modes: the stdout banner is what editor
integrations parse, and a warning interleaved into it would change that surface. Under
`--format json` the warning is a JSON object on stderr, so a machine consumer gets a
parseable signal rather than prose it must pattern-match.

Reporting is non-fatal by design. `watch` is a long-running development loop; one bad path
out of several should not end the session. The empty set stays fatal, unchanged.

## Claiming a pass

`run_check` forks `specsync check` and reads the child's exit status. That status cannot
distinguish the two cases that matter here, because a check that finds no specs exits 0 —
correctly, since bare mode is informational and `--strict` is where it gates (#560).

So the parent inferred success from the absence of failure. The fix inverts the inference:
`watch` claims a pass only on **positive evidence** that specs were examined. The child's
"no spec files found" line is that evidence, read on the stream watch already scans for the
failed-spec count, so no second pass over the output and no new coupling.

The alternative — parsing the `N specs checked:` summary and testing `N > 0` — was
rejected: the summary is absent on paths that legitimately examine specs incrementally, so
its absence would produce a false "nothing was checked" on ordinary re-runs.

## What is deliberately not touched

`check` itself. It was already correct: `check --strict` fails this tree and drill 060
keeps a control asserting that #560/#582 still hold. The defect was `watch` narrating a
check it had not evaluated.
