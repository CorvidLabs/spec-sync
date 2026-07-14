## ADDED

### REQUIREMENT REQ-validator-006

Default source discovery SHALL include `.mjs` and `.cjs` files in strict file and LOC coverage denominators.

Acceptance Criteria

- Mapped module files increase measured file and LOC totals using their real contents.
- An uncovered `.mjs` or `.cjs` file prevents strict 100 percent coverage from passing.
- Coverage output reports non-vacuous exact totals for mixed default-language projects.
