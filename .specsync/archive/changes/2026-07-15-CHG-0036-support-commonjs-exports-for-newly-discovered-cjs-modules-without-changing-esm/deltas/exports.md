## ADDED

### REQUIREMENT REQ-exports-002

The TypeScript/JavaScript export scanner SHALL report statically named CommonJS exports in both regex and AST modes without changing existing ESM results.

Acceptance Criteria

- `exports.foo = ...` and `module.exports.foo = ...` report `foo`.
- Top-level shorthand and named keys in `module.exports = { ... }` are reported.
- Comments, strings, computed keys, and statically unresolved spreads do not report false exports.
- Mixed ESM/CommonJS input is stable and deduplicated in both parsing modes.

### SPEC SECTION CommonJS Extraction

The TypeScript/JavaScript backend supplements its ESM and TypeScript `export =` handling with static CommonJS name discovery in both regex and AST modes.

- Direct `exports.foo = value` and `module.exports.foo = value` assignments report `foo`.
- Top-level shorthand, named properties, and identifier-named methods in `module.exports = { ... }` report their static keys.
- Comments, string and template literals, computed keys, and unresolved spreads never create exports.
- Mixed ESM/CommonJS names are stable and deduplicated without executing source code.

For `exports.foo = 1; module.exports = { bar, baz: value, [dynamic]: value, ...extra };`, both parsing modes report `foo`, `bar`, and `baz`, while ignoring `dynamic` and `extra` because their exported names are not statically determined.
