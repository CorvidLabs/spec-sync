---
change: split-mcp-issue-verification-into-focused-private-modules
artifact: testing
---

# Testing

Baseline: cargo build --locked passed with Rust 1.89.0. cargo test --locked mcp:: passed 124 unit tests and 72 integration tests. The test build reports a pre-existing unused_mut warning in src/change_tests.rs:3803, outside this scope.

For the refactor, compare moved function bodies and test names/cfg guards to the base, then run cargo fmt --check, cargo check --locked, cargo test --locked mcp::, and specsync check --strict --require-coverage 100 --force. Run the full available verify steps and report exact outcomes. Keep Linux-only and Unix-only tests enabled on this Linux host; do not claim Windows runtime validation.

The tool catalog and mcp diff are unchanged by design. Do not refresh the lock as a substitute for checking it. fledge trust verify and Attest remain required before claiming complete trust; their availability must be checked independently of cargo test.

## Executed Refactor Evidence

- Exact production function bodies match the base byte-for-byte except parent-only handler visibility. All 17 test bodies and cfg guards match after rustfmt; a Linux-only fixture import is cfg-gated following independent review.
- cargo fmt --check, cargo check --locked, and cargo clippy --locked -- -D warnings passed.
- Final focused run passed 124 MCP unit tests and 72 MCP integration tests, including the 17 moved tests.
- Strict source/spec validation passed 62 specs with zero warnings and 108/108 covered source files.
- The tool catalog lock comparison passed without rewriting the lock.
- Release-candidate validator passed all 52 Python tests.
- Full unit suite before and after the move: 2327 passed, 184 failed, with identical failing test names. 174 failures directly report ambiguous Git discovery because the sandbox exposes an ancestor /tmp/.git marker. No security check was weakened. The remaining baseline failures are also unchanged; the full suite is not green.
- Required CI-gate tests stopped with 9 errors because Ruby/Psych is unavailable; fledge trust verify could not start because fledge is unavailable. No trust or provenance pass is claimed.
- Independent source review has no remaining findings after gating the Linux-only ProjectSnapshot fixture import. Non-Linux runtime tests were not run.

- cargo build --release --locked passed on the final production source.
