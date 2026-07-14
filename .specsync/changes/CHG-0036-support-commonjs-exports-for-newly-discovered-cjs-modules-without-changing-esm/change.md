---
id: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
state: accepted
type: feature
base_commit: 096425ec9fc32e58f18aa2d42a7c65a30fac41cf
---

# Support CommonJS exports for newly discovered .cjs modules without changing ESM behavior

## Intent

Support CommonJS exports for newly discovered .cjs modules without changing ESM behavior

## Affected Canonical Specs

- `exports`

## Acceptance Criteria

- Regex and AST modes extract direct CommonJS property assignments,Regex and AST modes extract statically named top-level object export keys while ignoring unresolved syntax,Existing TypeScript and ESM extraction remains unchanged and deduplicated,Focused regressions plus complete tests strict specs hosted CI and Trust pass

## No-spec Rationale

Not applicable
