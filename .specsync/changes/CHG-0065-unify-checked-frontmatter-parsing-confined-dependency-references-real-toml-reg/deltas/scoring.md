## ADDED

### REQUIREMENT REQ-scoring-002

Spec quality scoring SHALL use checked frontmatter and typed dependency resolution without probing
outside the project or penalizing valid non-path references.

Acceptance Criteria

- Malformed frontmatter or dependency references produce an invalid or zero-gating score with an
  actionable, non-secret diagnostic and receive no freshness credit.
- Valid bare modules, registry-mapped paths, and remote references are not treated as missing
  project-root paths.
- Missing or unsafe local dependencies receive the same normalized finding as check, deps, and
  resolve.
- Scoring never opens an absolute, traversing, or symlink-escaping dependency target and does not
  leak outside-root identifiers in suggestions or MCP output.

## MODIFIED

### SPEC SECTION Invariants

1. A valid total score is 0 through 100, composed of five components worth 0 through 20 each.
2. Grade scale remains A for 90-100, B for 80-89, C for 70-79, D for 60-69, and F below 60.
3. Frontmatter scoring uses checked parsing; malformed or ambiguous frontmatter produces an invalid
   zero-gating result with actionable diagnostics and no freshness credit.
4. Dependency freshness consumes shared typed and confined resolution; valid bare modules,
   registry-backed custom paths, and remote references are not treated as project-relative files.
5. Missing, malformed, identity-conflicting, or unsafe local dependencies use the same normalized
   finding and exact raw declaration as check, deps, and resolve.
6. Scoring never probes absolute, traversing, backslash, drive/UNC, or symlink-escaping dependency
   targets and never exposes outside-root content in suggestions or MCP results.
7. Unfinished-work marker counting ignores fenced examples and counts only standalone markers.
8. Content depth requires meaningful content beyond headings, comments, and separator rows.
9. No exports to document earns full API score, so configuration-only modules are not penalized.
10. Suggestions remain actionable and `SpecScore.explain` remains populated with one deterministic
    breakdown per dimension.
