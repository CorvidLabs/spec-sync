## MODIFIED

### REQUIREMENT REQ-cmd-check-001

Unified checking SHALL preserve the documented top-level schema and complete originating-spec
diagnostics when SDD validation, checked frontmatter, dependency resolution, registry loading, or
coverage discovery fails.

Acceptance Criteria

- Failed structured output includes `passed`, `errors`, `warnings`, `stale`, and `specs_checked`.
- Structured SDD detail remains available as an additive field.
- Malformed manifest discovery exits nonzero and emits valid JSON with `passed: false`,
  `valid: false`, `inconclusive: true`, and an explicit error.
- Checked-frontmatter diagnostics are attributed to the originating spec and malformed specs are
  counted rather than omitted.
- Malformed, missing, unsafe, identity-conflicting, or registry-failing dependencies are hard
  findings with exact raw declarations and normalized identities when available.
- JSON remains the only stdout payload on every parser, registry, dependency, or coverage failure.

### SPEC SECTION Invariants

1. `--fix` performs deterministic local Markdown repairs only and never invokes a model or shell
   inference command.
2. Near-miss header correction and undocumented-export repair preserve existing Public API tables
   and never add a symbol already documented in any Public API table.
3. Hash cache bypass rules remain explicit for force, strict, fix, and filtered runs; fixes are
   revalidated before a successful result.
4. Checked-frontmatter failures remain attributed errors and cannot remove a spec from counts,
   structured output, cache decisions, or the final verdict.
5. Dependency validation uses the shared typed, registry-aware, confined resolver; malformed,
   missing, unsafe, registry-load, and module-identity failures are hard findings independent of
   `--strict`.
6. Remote dependency syntax is checked locally without initiating network access.
7. JSON output is one complete machine-readable object containing all parser, registry,
   dependency, SDD, and coverage failures before exit.
8. `--create-issues` groups findings by originating spec and creates no more than one drift issue
   per affected spec.
9. `--explain` retains deterministic per-category score breakdowns without converting invalid
   checked input into freshness credit.
10. Exit status uses the shared outcome contract; findings and inconclusive requested gates exit 1,
    usage errors exit 2, and `--strict` only promotes advisory warnings.
11. Coverage uses checked manifest discovery and cannot report partial, malformed, unreadable, or
    zero-source requested coverage as a successful gate.
