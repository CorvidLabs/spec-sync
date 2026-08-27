---
change: a-configured-source-dirs-must-survive-a-manifest-discovery-failure-and-an-in-repo-includebuild-must-be-judged-by-its
artifact: requirements
---

# Requirements

Two defects, one class. An input the tool could not interpret was treated as a verdict about the
project, and the project's own explicit statement could not overrule it.

## R1 — A stated `source_dirs` wins over a failure to infer one (the primary fix)

Manifest discovery exists to INFER a source list the project did not state. When the project HAS
stated one, a failure to infer it is not a finding about the project and must not abort the command.

- `compute_coverage_checked` completes over the configured list when discovery fails and
  `source_dirs` was explicitly configured. `check` and `coverage` therefore run to completion.
- The failure is disclosed as a coverage notice carried with the figures — not to stderr, and not
  in place of them. Manifest modules seed module attribution, so a degraded run reports fewer
  modules without specs than the tree holds; the number cannot be read apart from what shaped it.
- The failure REMAINS fatal when `source_dirs` was not configured. There the list coverage would
  measure is itself discovery output, so degrading would report a percentage over a guess.
- "Explicitly configured" is what the config file said, recorded at load time. It cannot be
  recovered afterwards: a configured `["src"]` and the `["src"]` default are the same list.

## R2 — `includeBuild` is judged by its argument, not its token

- `includeBuild("vendor/podo-shared")` — one complete literal path confined beneath the project
  root — parses. It contributes no module: a composite build is a separate build, and the root
  build's own `include(...)` list is what modules come from.
- `includeBuild("../outside")`, rooted/absolute paths, interpolated or otherwise dynamic
  expressions, more than one argument, and a trailing configuration block all keep failing closed.
- The refusal names the offending ARGUMENT rather than the token, so an escape can be told apart
  from a form the parser does not model.
- `includeFlat` and `includeWorkspace` are unchanged. `includeFlat` resolves against the PARENT of
  the root, so its argument is outside the project by construction; `includeWorkspace` is not a
  form this parser models. Reading their arguments would not make either supportable.

## Out of scope

Discovering modules or source directories from INSIDE an accepted composite build. Its settings are
a separate build's manifest; parsing them is a feature, not this fix. Accepting and ignoring is
strictly better than aborting and is what unblocks the reported project.
