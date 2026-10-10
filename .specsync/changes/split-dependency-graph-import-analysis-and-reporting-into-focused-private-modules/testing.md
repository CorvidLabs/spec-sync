---
change: split-dependency-graph-import-analysis-and-reporting-into-focused-private-modules
artifact: testing
---

# Testing

Preserve all 43 named tests, their bodies, literal contents, assertions and attributes. Compare the original inventory from the main baseline unit log against the refactored compiled inventory, allowing owner module relocation only. Compare every moved production body, type, constant and scanner expression against the saved original.

After the parent grants the shared Cargo target, run cargo test --bin specsync deps::, the focused compiled --list inventory, cargo fmt --check, production cargo clippy --bin specsync -- -D warnings, cargo check, cargo check --release, cargo build --release, strict spec checking with 100 percent file coverage, the repository pre-push gate and real scoped change check --commit. The parent will run combined full unit/integration baseline comparisons after both parser and deps checkpoints; focused validation is not a full-suite claim. Record Fledge/Attest and other unavailable tools without weakening gates.

## Recorded verification

- Original source inventory: all 43 deps tests passed in the main 815690d baseline unit run. No duplicate pristine build was run.
- Refactored focused run: `cargo test --bin specsync deps::` passed 43 tests, zero failed or ignored. The compiled `--list` inventory matches the original 43-name multiset exactly after owner-module relocation.
- Complete graph, type, regex, scanner/ownership and report sections match the original bytes apart from the minimal `pub(super)` cooperation seam. All 43 complete test blocks and both fixture builders match rustfmt-normalized original blocks, including literal contents.
- Independent read-only review found no issues: all 78 function-body token sequences, complete test declarations, types, constants, statics, impls and existing attributes are preserved. Source-root guards, fail-loud reads, ImportDialect authority, JVM attribution and Rust masking were reviewed explicitly. Two new lint allowances are scoped solely to facade reexports unused by this binary.
- `cargo fmt --check`, production `cargo clippy --bin specsync -- -D warnings`, debug and release `cargo check`, and `cargo build --release` passed.
- Strict forced validation passed all 62 specs without warnings; file coverage is 113/113 (100 percent). The repository pre-push script passed when invoked with bash; its existing non-executable mode was not changed. The deps spec scores 100/100.
- The focused test build retains only the pre-existing `src/change_tests.rs:3813` unused-mut warning. No unrelated warning was changed.
- Full isolated unit/integration suites were not repeated. The parent will run combined baseline comparisons after the parser and deps checkpoints. The existing main baselines report 2338 unit passes / 192 failures and 425 integration passes / 13 failures caused by sandbox Git fixtures; these are not claimed as passing here. All-target lint has 31 known baseline diagnostics; production lint is green.
- Fledge and Attest are unavailable in this environment (`fledge trust verify` and `attest --version` cannot start). Ruby and Windows checks were not run. CI/trust and human implementation review remain required before shipping. No gate was weakened, no permission workaround was used, and nothing was pushed or accepted.

- The actual scoped `change check` materialized the approved internal-organization delta and passed `specsync check --spec deps`. Its generated extra EOF blank line was trimmed before the final `check --commit` sequence.
