---
change: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
artifact: context
---

# Context

`deps` silently skipped Kotlin import analysis. On a fixture with real Kotlin
imports it reported `✓ All dependency declarations are valid` with rc=0, having
collected nothing — zero edges for want of a parser, reported as zero problems.

A first attempt added the Kotlin extractor and closed its gate. It was judged
INCOMPLETE because it reproduced the same defect one layer down:
`resolve_kotlin_package` matched an import's package prefix only against
directory SUFFIXES, and `filter_map` dropped anything unresolved with no record.
A Kotlin file whose directory does not literally end with its full package path
yielded zero edges, `undeclared_imports: []`, and exit 0.

So the fix for "zero edges reported as zero problems" reported zero edges as
zero problems. It collected the imports and then discarded the ones it could not
map, presenting a dependency graph built from a resolution that failed.

The reviewer also found the disclosure it added fired for languages with no
import concept at all: a pure-Rust project listing `src/ci.yml` and
`src/tool.sh` printed "Import analysis is not implemented for Bash, Yaml" and
downgraded its verdict. A YAML file has no imports to miss.
