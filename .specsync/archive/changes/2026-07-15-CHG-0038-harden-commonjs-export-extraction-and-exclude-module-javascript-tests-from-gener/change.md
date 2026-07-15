---
id: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
state: archived
type: bug_fix
base_commit: 044eb584769487b8c43f1509a22d1693893d9894
---

# Harden CommonJS export extraction and exclude module JavaScript tests from generated specs

## Intent

Harden CommonJS export extraction and exclude module JavaScript tests from generated specs

## Affected Canonical Specs

- `exports`
- `cmd_new`
- `cmd_scaffold`

## Acceptance Criteria

- Chained CommonJS property assignments report every static export without phantom names from function-local aliases or regular-expression literals.
- Type-level scans preserve statically exported CommonJS classes in regex and AST modes.
- New and scaffold omit recognized JavaScript-family test files while retaining production module sources.
- All existing ESM TypeScript CommonJS and generation behavior remains green in the complete native and hosted matrices.

## No-spec Rationale

Not applicable
