---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: context
---

# Context

#562 was reported as "coverage displays 0/0 as 100%". It was fixed in
`src/output.rs` and shipped as #575. That fix was correct and it reached one of
nine sites.

The other eight were invisible because each is only reachable through a format
or a transport the original repro never used: `report` in any format,
`coverage --json`, and both MCP surfaces. Text told the truth while JSON, CSV,
Markdown and MCP all still said the project was fully covered.

The root is not any of the nine expressions. It is that `CoverageReport` carried
the answer as a precomputed `usize`:

    let coverage_percent = if total_files == 0 { 100 } else { ... };   // validator.rs:5257
    let loc_coverage_percent = (specced_loc * 100).checked_div(total_loc).unwrap_or(100);

Every consumer that read the field inherited the lie — fourteen of them,
including the `--require-coverage` gate at `commands/mod.rs:1223`, which
compared `100 < req` and passed. `src/output.rs` was the only consumer that got
it right, and only because it ignored the field and re-derived from the counts.
That is the tell: a field nobody can trust is a field with the wrong type.

The contradiction this produced was visible inside a single run.
`report --require-coverage 80` over a zero-source tree exited 1, because
`compute_exit_code` inspects `total_source_files` directly, while the same run's
JSON said `"coverage_percent": 100`. Two mechanisms disagreeing about one tree,
because one read the counts and the other read the field.

Ruled out: fixing the eight remaining expressions in place. That leaves the
field wrong, leaves the fourteen readers wrong, and leaves the next caller free
to write the division a tenth time. It was tried, in a sense — #575 was that
approach applied once.
