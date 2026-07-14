---
change: CHG-0036-support-commonjs-exports-for-newly-discovered-cjs-modules-without-changing-esm
artifact: context
---

# Context

CHG-0035 adds `.cjs` to default TypeScript-family discovery and strict coverage denominators. The current TypeScript export scanners recognize ESM declarations and TypeScript's `export = Name`, but ordinary CommonJS assignments can therefore be discovered without their public symbols being extracted.

This change closes that contract gap without broadening file discovery or altering ESM behavior. A shared lexical CommonJS helper will supplement both the regex and AST paths so mixed ESM/CommonJS input and AST-success cases have the same deterministic output.

The scanner intentionally recognizes only statically named exports. Dynamic computed keys and unresolved spreads are outside the contract because their exported names cannot be determined without executing user code.
