---
change: split-mcp-integration-regressions-into-cohesive-topical-test-modules
artifact: testing
---

# Testing

Save pristine source and the 72-name Linux compiled inventory before edits. Compare all 86 complete function declarations including bodies, fixture literals, signatures, visibility, and attached cfg/cfg_attr attributes after movement; verify all 77 test declarations, all four release-ignore attributes, and every private child registration. Run cargo test --locked --test integration mcp:: and the unchanged every_integration_test_file_is_registered guard. Compare the complete integration result against the known 425-pass/13-failure baseline by exact failing test names.

Run cargo fmt --check, cargo check --locked --test integration, production and integration clippy with warnings denied, strict 100% source coverage and MCP spec quality, the mandatory pre-push gate, and real change check --commit. Confirm no production source, sibling test, root harness, or shared helper changes. Full unit tests are not repeated for pure test movement. Missing trust/provenance tools, known baseline lint/test failures, and unavailable native Windows runtime must remain explicit. Independent agent technical review is not human implementation approval.

## Verified Movement and Focused Results

The registered facade shrank from 4139 to 92 lines. Its nine private children contain 156–703
lines: configuration 403, gradle 415, issues 156, limits 352, protocol 483, read_roots 569,
snapshots 703, tools 454, and workspaces 548. All 86 complete original declarations match exact
Rust AST tokens, including signatures, visibility, bodies, literals, and attached attributes.
A separate original-line partition confirms every original line is assigned exactly once and
every moved segment remains byte-for-byte present in its destination, including fixture strings
and explanatory comments. No rustfmt exception is needed.

All 77 original tests remain: 51 unrestricted, 19 Unix-only, five Windows-only, and two
Unix-or-Windows. The four release-ignore attributes and the Windows command-helper import are
unchanged. Original and moved Linux compiled inventories contain the same 72 leaf names exactly
once. All 72 focused MCP tests pass both before and after movement; the unchanged root
every_integration_test_file_is_registered guard passes. cargo fmt --check and
cargo check --locked --test integration pass. No production source, sibling integration file,
root harness, or shared integration helper changed.

## Complete Integration Comparison

The complete refactored integration suite has 425 passes and 13 failures. The exact failing-name
set matches the supplied pristine-main 815690d baseline. Every MCP case passes. The existing
lifecycle/comment Git-fixture failures were neither edited nor suppressed. Production
cargo clippy --locked -- -D warnings passes; stricter integration/all-target lint evidence follows below.

## Lint and Environment Boundaries

cargo clippy --locked --test integration -- -D warnings remains blocked by the existing
collapsible_if diagnostic at tests/integration/change.rs:19:5, reproduced on pristine main.
The all-target command reports 31 diagnostics whose messages and primary source locations
match the pristine-main baseline exactly; none is new in this split. No warning was suppressed.

The strict spec check passes all 62 specs with zero warnings and 107/107 production source
files covered, distinct from the separately compared compiled test inventory. MCP quality
is 100/A. Production unit tests were not rerun because production source is unchanged.
fledge trust verify cannot run because fledge is not installed. Attest and Ruby/Psych are
unavailable, and only the Linux Rust target is installed. Native Windows compilation/runtime
and complete repository trust/provenance qualification are not claimed. The Windows helper
import and complete guarded function declarations receive static review.

## Available Final Gates and Review

The mandatory pre-push gate passes, including formatting, production type checking, and strict
100% source/spec coverage. The approved documentation delta was materialized through the real
change check command, whose scoped MCP spec verification passed. The final canonical MCP spec
advances from version 30 to 31 and records this internal topology change in its changelog.

Independent technical review regenerated the original source from main 815690d and confirmed
exact AST equality and byte-preserving movement without relying on this implementation's saved
inventories. It found no source issue: every child is registered, helpers remain private at their
nearest shared owner, all platform/debug guards remain, and FIFO/socket semantics are unchanged.
This local checkpoint still requires genuine human implementation review and later qualified
trust/provenance/CI evidence; it is not Accepted or archived.
