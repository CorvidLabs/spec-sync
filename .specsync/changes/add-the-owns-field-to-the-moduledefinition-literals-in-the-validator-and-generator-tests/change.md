---
id: add-the-owns-field-to-the-moduledefinition-literals-in-the-validator-and-generator-tests
state: draft
type: refactor
base_commit: 404fe4d6fcef380d3675bab5cc1d2d4786d0401c
---

# Add the owns field to the ModuleDefinition literals in the validator and generator tests

## Intent

Add the owns field to the ModuleDefinition literals in the validator and generator tests

## Affected Canonical Specs

- `validator`
- `generator`

## Acceptance Criteria

- `src/validator.rs` and `src/generator.rs` compile with the new `owns` field present on every `ModuleDefinition` literal in their tests, and no validator or generator behaviour, contract, or public export changes

## No-spec Rationale

Three test-only struct literals gain the new owns field so the crate compiles; validator and generator behaviour, contracts, and public API are unchanged
