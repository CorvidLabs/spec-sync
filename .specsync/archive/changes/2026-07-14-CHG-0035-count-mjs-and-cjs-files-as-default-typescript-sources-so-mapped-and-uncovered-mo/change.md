---
id: CHG-0035-count-mjs-and-cjs-files-as-default-typescript-sources-so-mapped-and-uncovered-mo
state: archived
type: bug_fix
base_commit: a9422aedbe12a3c50787c1fcc074749232f25dfe
---

# Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators

## Intent

Count mjs and cjs files as default TypeScript sources so mapped and uncovered module files contribute to strict file and LOC coverage denominators

## Affected Canonical Specs

- `types`
- `validator`

## Acceptance Criteria

- Language::from_extension classifies mjs and cjs as TypeScript and TypeScript::extensions includes both suffixes.
- A default-discovery fixture mapping ts, css, mjs, and cjs files reports exact file and non-zero LOC denominators instead of omitting module JavaScript.
- Default-discovery fixtures with uncovered mjs or cjs files each fail strict require-coverage 100 and report those files in the denominator.
- Existing JavaScript and TypeScript discovery remains compatible, canonical deltas and changelog are accurate, and the complete native and hosted gates pass.

## No-spec Rationale

Not applicable
