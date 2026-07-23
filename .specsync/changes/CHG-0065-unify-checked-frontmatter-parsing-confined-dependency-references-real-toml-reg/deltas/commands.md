## ADDED

### REQUIREMENT REQ-commands-003

Shared command outcomes SHALL distinguish trustworthy completion, findings, inconclusive gates,
and usage errors before rendering or exiting.

Acceptance Criteria

- Trustworthy or advisory success maps to exit 0, findings and inconclusive gates map to exit 1,
  and usage errors map to exit 2.
- JSON serializes the complete result before exit and is the only stdout payload in JSON mode,
  including parser, registry, coverage, fetch, and resolution failures.
- Check, deps, resolve, scoring, and MCP integration fixtures assert the same dependency category,
  normalized identity when available, and exact raw spelling for the same declaration.

## MODIFIED

### SPEC SECTION Invariants

1. `load_and_discover` excludes underscore-prefixed internal or template specs.
2. `filter_specs` matches exact paths, project-relative paths, filename stems, and module names.
3. `run_validation` applies global, inline, and per-spec ignores before counting advisory warnings.
4. Draft validation notices remain separate from warnings in every renderer and do not fail strict
   enforcement by themselves.
5. Failing checks use unambiguous negated labels and never pair a failure glyph with a passing
   description.
6. Shared outcomes classify trustworthy or advisory completion as exit 0, findings or
   inconclusive requested gates as exit 1, and usage errors as exit 2 before rendering.
7. A requested coverage gate is evaluated through checked coverage; malformed discovery,
   unreadable configured inputs, or zero discoverable sources is inconclusive rather than
   vacuously successful.
8. JSON serializes the complete result before exit and is the only stdout payload in JSON mode,
   including parser, registry, coverage, transport, and resolution failures.
9. Check, deps, resolve, scoring, and MCP preserve the same dependency category, normalized
   identity when available, and exact first raw spelling for equivalent declarations.
10. `create_drift_issues` groups errors by originating spec and creates at most one issue per spec.
