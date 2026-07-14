---
change: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| Direct assignments | Unit cases for `exports.foo` and `module.exports.foo` in regex and AST modes |
| Object exports | Unit cases for shorthand, named properties, methods, and nested values |
| False-positive resistance | Unit cases covering comments, strings, computed keys, and unresolved spreads |
| Compatibility | Existing TypeScript/ESM suites plus mixed ESM/CommonJS deduplication cases |
| Repository integrity | Complete native tests, `specsync check --strict`, hosted CI, `fledge trust verify`, and Attest verification |

Focused commands:

- `cargo test exports::typescript::`
- `cargo test exports::ast::typescript::`

The lifecycle verification command and its exact evidence will be recorded before closing approval.

Canonical requirement evidence:

- `REQ-exports-002`: `test_commonjs_direct_and_object_exports`, `test_commonjs_ignores_non_static_and_non_code_names`, `test_commonjs_mixed_with_esm_is_deduplicated`, and `test_commonjs_exports_match_static_contract` prove static extraction, false-positive resistance, deduplication, and regex/AST parity.
