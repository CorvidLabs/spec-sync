---
change: split-command-integration-regressions-into-cohesive-topical-test-modules
artifact: testing
---

# Testing

Baseline pristine-main integration log contains 120 passing commands:: cases; the full integration suite has 425 passes and 13 known lifecycle/comment failures. Save the original declaration/body/attribute inventory and mechanically compare every original function and literal byte after movement, explicitly reviewing rustfmt-only punctuation changes. Run cargo test --locked --test integration commands:: and the full integration suite; compare exact failing names for unchanged modules and short test-name inventory for moved command tests. Confirm the root every_integration_test_file_is_registered guard passes. Run cargo fmt --check, production clippy with warnings denied, targeted integration clippy where possible, strict spec coverage, mandatory pre-push and real change check --commit. No production source changes are allowed. Ruby/Psych and fledge/Attest remain unavailable; do not claim full trust or Windows runtime. Independent technical review is not human implementation approval.

## Focused Refactor Evidence

The registered facade is reduced from 5496 to 103 lines. Its 11 explicitly declared children are 185–970 lines. All 134 complete original function declarations match exactly, including signatures, visibility, bodies, literals and attached cfg/cfg_attr attributes; no rustfmt exception or added function is needed. All 124 test declarations remain, including four Windows-only cases. The compiled Linux inventory contains the same 120 short test names with no additions or omissions, and all 120 focused cases pass. The unchanged root registration guard passes. cargo fmt --check and cargo check --locked --test integration pass. Working-tree diff checks passed; the full committed-patch fixture diagnostic is explained below.

The Windows commands::create_windows_junction helper retains its original path and visibility for the sibling MCP integration module. All other integration files, shared helpers, root harness and production source remain unchanged. Native Windows runtime is not claimed.

## Full Integration and Lint Evidence

The refactored complete integration suite has 425 passes and 13 failures. The exact failing-name set matches pristine main 815690d; no moved command test fails. Production cargo clippy --locked -- -D warnings passes. The stricter cargo clippy --locked --test integration -- -D warnings is blocked by clippy::collapsible_if at tests/integration/change.rs:19. A fresh run on pristine main reproduces that same error. The unrelated lifecycle fixture was not edited or suppressed.

Independent final technical review confirms all declarations and bodies, all 120 compiled Linux names exactly once, all 11 child registrations, the unchanged root registration guard, and the Windows MCP junction seam. No production code, root harness, shared helpers or sibling integration module changed. The full unit suite was not rerun for this test-only movement; it has no production-source diff. Ruby/Psych, fledge/Attest and native Windows runtime qualification remain unavailable, so full trust is not claimed.

## Final Available Gates

The release build and mandatory pre-push gate passed. Strict validation reports all 62 specs passing, zero warnings and 107/107 production source files covered; this source-coverage figure is distinct from the separately verified compiled test inventory. CLI spec quality is 100/A. All available checks and baseline limitations are recorded above, and the local result is prepared for the real scoped lifecycle materialize/verify commit. No human implementation review, publication or merge is claimed.

## Intentional Merge-Fixture Diff Diagnostic

`git diff --check origin/main..HEAD` reports six conflict-marker lines in the relocated maintenance.rs file. They are the original literal fixture contents inside merge_never_writes_corrupt_spec_for_fence_hunk and merge_resolves_interior_field_conflict, preserved exactly by the 134-body comparison. They are deliberately conflicted inputs tested by the merge command, not unresolved repository merges. The index has no unmerged entries (`git ls-files -u` is empty), the Rust source compiles and the fixtures pass. No whitespace error is reported. The strings and Git checks were not altered or suppressed to hide this expected diagnostic.
