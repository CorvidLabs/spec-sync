---
change: split-validator-validation-and-retained-coverage-into-focused-private-modules
artifact: testing
---

# Testing

Capture the baseline validator test inventory and results before source edits. Preserve every production function body and complete literal bytes; review any forwarding facade separately. Preserve all existing tests, fixture behavior and platform guards; compare normalized bodies and names, then run focused validator tests and affected check/coverage/MCP integration regressions. Run cargo fmt --check, cargo check --locked, cargo clippy --locked -- -D warnings, cargo build --release --locked, strict 100 percent spec coverage and the mandatory pre-push gate. Use the real scoped change check --commit. Compare full-unit failure names with pristine latest main; do not weaken Git-discovery or sandbox checks. Ruby/Psych and fledge/Attest remain unavailable unless independently established otherwise. Linux runs do not qualify Windows runtime. Independent review and genuine human implementation review are distinct.

## Baseline Evidence

At pristine main 815690d, the exact compiled unit-test binary passes all 64 validator tests. The full unit suite has 2338 passes and 192 failures; the complete integration suite has 425 passes and 13 failures. Preserve and compare failure-name sets rather than treating focused green checks as full-suite success. These baselines were run before validator product edits.

## Focused Refactor Evidence

- Facade reduced from 6416 to 257 lines; all 24 Rust files are below 1000 lines. Largest production file is 701 lines, largest test file 699.
- Preserved 65 existing test declarations (64 on Linux, one Windows-only) and four shared fixtures. Focused cargo test --locked validator:: passed all 64 Linux tests.
- Debug and release cargo check passed. cargo clippy --locked -- -D warnings passed.
- All 177 original function/method/test bodies match after formatting in the same rustfmt context, with complete literal bytes retained. Raw-token comparison separately exposes one optional trailing comma removed from a relocated closure-call test; no production body changed.

- Independent final technical review found no blocking issues: signatures, attributes, types/constants, API visibility, snapshot-only paths, root/budget lifetimes, checkpoint ordering, notices, platform guards and test wiring were checked. It confirms the same 64 Linux test names after module-path relocation. This is not a human implementation acceptance.

## Final Available Gates

Release build passed. Strict coverage passed all 62 specs with zero warnings and 118/118 covered source files. Validator spec quality is 100/A. The mandatory pre-push gate passed.

The refactored full unit suite has 2338 passes and 192 failures; the complete integration suite has 425 passes and 13 failures. Both failure-name sets exactly match the pristine main baseline. No security or Git-discovery guard was changed to obtain a pass. The full suite is not green in this environment; missing Ruby/Psych and fledge/Attest prevent claiming their CI/trust qualification. Windows runtime was not executed.

Source and documentation are prepared for the real scoped lifecycle materialize/verify commit. No human implementation review, publication or merge is claimed.
