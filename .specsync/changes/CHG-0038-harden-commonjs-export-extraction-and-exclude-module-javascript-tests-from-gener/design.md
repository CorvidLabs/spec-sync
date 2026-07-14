---
change: CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener
artifact: design
---

# Design

Keep CommonJS extraction static and deterministic. Property matching must end at the
assignment operator so chained assignments remain independently discoverable. Mask
regex literals before scanning, and reject matches nested inside function-like AST
scopes so parameter/local aliases are not treated as module exports. Type filtering
intersects exported names with ordinary and inline CommonJS class declarations.

`new` and `scaffold` reuse `exports::is_test_file` at file discovery time,
matching the coverage denominator policy without changing configured extensions.
