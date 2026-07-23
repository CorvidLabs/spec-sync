## ADDED

### REQUIREMENT REQ-validator-009

`specsync check` SHALL fail closed on checked-frontmatter, dependency-reference, registry, module
identity, and confinement errors.

Acceptance Criteria

- Every checked-frontmatter diagnostic is surfaced against the originating spec and is an error in
  strict validation; malformed specs cannot disappear from the result set.
- Every local dependency is resolved through the shared typed resolver; missing, malformed,
  absolute, traversing, and symlink-escaping references are errors containing the offending text.
- Every remote dependency is syntactically validated during local check even when network
  verification is not requested.
- For a conventional `specs/<module>/<module>.spec.md` path, frontmatter `module` equals the
  canonical module identity; registry-mapped custom paths use the registry mapping as authority.
- Check, deps, resolve, score, and MCP fixtures assert identical normalized dependency identities
  and compatible diagnostics for the same malformed or unresolved reference.

## MODIFIED

### SPEC SECTION Invariants

1. Validation is bidirectional: documented nonexistent exports are errors and undocumented code
   exports are warnings.
2. Checked-frontmatter diagnostics are errors attached to the originating spec; malformed specs
   are never omitted from validation results.
3. Missing required frontmatter fields are errors, and known fields retain their checked scalar,
   sequence, issue-number, and version contracts.
4. Every local dependency uses the shared typed and root-confined resolver; malformed, missing,
   absolute, traversing, backslash, identity-conflicting, and symlink-escaping references are
   errors retaining exact raw spelling.
5. Remote dependencies are syntax-checked locally and require no network unless remote resolution
   is explicitly requested.
6. Conventional `specs/<module>/<module>.spec.md` locations require matching frontmatter module
   identity; registry-mapped custom locations use the registry module key as authority.
7. Coverage excludes tests and configured exclude patterns, respects configured source extensions,
   and handles supported simplified glob forms without panicking.
8. `find_spec_files` returns sorted results, schema extraction uses the configured pattern, and
   missing-file suggestions use Levenshtein distance with a maximum of three.
9. Flat source files are detected as modules except common entry points.
10. Sections without substantive content are reported as unfinished draft text.
11. `validate_spec` records the parsed lifecycle status or `None` when checked parsing fails, so
    reporters can surface status-dependent behavior without suppressing parse diagnostics.
12. Requirements companions remain optional under adaptive policy but are validated when present.
13. `compute_coverage_checked` propagates malformed or unreadable discovery as inconclusive; the
    compatibility `compute_coverage` wrapper cannot be used by a gate to claim success.
14. Check, dependency graph, resolution, scoring, and MCP use the same dependency categories,
    normalized identities, and raw declarations for equivalent findings.
