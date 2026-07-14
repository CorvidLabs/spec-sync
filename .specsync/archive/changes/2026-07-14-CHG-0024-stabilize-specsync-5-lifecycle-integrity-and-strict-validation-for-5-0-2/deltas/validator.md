## ADDED

### REQUIREMENT REQ-validator-002

Coverage SHALL measure configured static content without presenting a vacuous successful percentage.

Acceptance Criteria

- Mapped HTML reports one covered file out of one.
- Unmapped HTML reports zero covered files out of one and fails a 100 percent gate.
- Excluded assets remain excluded and static files require no exported symbols.
- A zero-file project is reported distinctly from measured 100 percent coverage.

### REQUIREMENT REQ-validator-003

Strict validation SHALL reject known unfilled companion scaffold markers with artifact-specific line diagnostics.

Acceptance Criteria

- Generated companion markers are recognized deterministically by artifact type.
- Concrete replacement prose passes.
- Similar prose and fenced examples are ignored.
- Diagnostics identify companion path line and required correction.

## MODIFIED

### SPEC SECTION Purpose

Core validation engine for spec-sync. Validates individual specs and selected companion artifacts against source code, discovers spec and source files including configured static content, extracts schema table names from SQL migrations, computes non-vacuous file and LOC coverage metrics, and resolves cross-project dependency references.
