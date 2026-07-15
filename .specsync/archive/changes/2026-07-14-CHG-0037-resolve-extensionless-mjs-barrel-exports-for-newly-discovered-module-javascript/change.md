---
id: CHG-0037-resolve-extensionless-mjs-barrel-exports-for-newly-discovered-module-javascript
state: archived
type: bug_fix
base_commit: dac64a1bff31cc5af0b590c8a800468440048e1e
---

# Resolve extensionless mjs barrel exports for newly discovered module JavaScript sources

## Intent

Resolve extensionless mjs barrel exports for newly discovered module JavaScript sources

## Affected Canonical Specs

- `exports`

## Acceptance Criteria

- An extensionless export-star barrel in an mjs source resolves a sibling mjs module and reports its public names.
- The resolver probes module JavaScript file and index variants without regressing existing TypeScript or JavaScript resolution.
- Strict validation passes for the regression fixture in both regex and AST parse modes.

## No-spec Rationale

Not applicable
