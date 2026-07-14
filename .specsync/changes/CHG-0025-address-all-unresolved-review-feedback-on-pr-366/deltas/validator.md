## ADDED

### REQUIREMENT REQ-validator-004

Strict validation SHALL discover default static projects and reject every unfilled marker emitted by built-in companion templates.

Acceptance Criteria

- Zero-config root and nested HTML, HTM, and CSS files select their containing source directory.
- Ignored directories remain excluded from static discovery.
- Every generated Layout, Components, Tokens, and Assets design marker produces an artifact-specific line diagnostic.
- Concrete replacements pass while fenced examples and similar prose remain ignored.

## MODIFIED

### SPEC SECTION Purpose

Core validation engine for spec-sync. Validates individual specs and selected companion artifacts against source code, discovers configured and zero-config source files including static HTML, HTM, and CSS content, rejects every known generated companion marker outside fenced examples, extracts schema table names from SQL migrations, computes non-vacuous file and LOC coverage metrics, and resolves cross-project dependency references.
