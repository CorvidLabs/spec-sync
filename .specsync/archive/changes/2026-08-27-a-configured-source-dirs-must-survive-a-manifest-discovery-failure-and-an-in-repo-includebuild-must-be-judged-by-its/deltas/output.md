## MODIFIED

### SPEC SECTION Invariants

1. Color thresholds for coverage: 100% = green, 80-99% = yellow, <80% = red
2. `print_summary` counts: passed is green, warnings is yellow, failed is red (failed = total - passed)
3. `print_diff_markdown` calls into `parser::parse_frontmatter` and `exports::has_extension` to cross-reference changed files against spec source file lists
4. Markdown output uses GitHub-flavored markdown with tables and emoji status icons (✅/❌/⚠)
5. All functions write to stdout via `println!` — no buffered or file output
6. Whatever shaped the coverage denominator is printed with the coverage figures, never in a
   separate section a reader may not reach: files referenced but missing, symlinks not
   traversed, and manifests that were degraded rather than allowed to abort the command. A
   degraded manifest still declared modules, so the module counts beside it were measured
   over less than the tree holds. Text, markdown, and JSON all carry them.
