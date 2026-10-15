---
change: complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1
artifact: testing
---

# Testing

Preserve the existing old optimized binary and recorded failing-before fixtures. Add real batch rerun tests, root-file creation process fixtures for all three creation commands, source selection unit tests for filters/overlap/ambiguity/explicit mappings and symlink/escape rejection, and watch selection/mode/diagnostic tests. Probe actual backend source/spec events with owned bounded processes; an absent positive control is an honest environmental limit, never a passing event test.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-cmd-generate-003 | Scoped batch first run/rerun text+JSON, flat-module control, byte-identical existing spec/companions. |
| REQ-generator-008 | Exact-file, overlap/dedup, ambiguous two-source, explicit mapping, extension/test/exclusion and confinement unit/process matrix. |
| REQ-cmd-new-005 | Root Cargo new produces truthful nonempty files and passing subsequent check; ordinary src directory retained. |
| REQ-cmd-scaffold-005 | Root Cargo add-spec and scaffold produce truthful source bindings and passing subsequent check, preserving existing companion handling. |
| REQ-watch-003 | Confined file/directory selection and registration mode; missing/invalid distinction; bounded actual source/spec event controls. |
| REQ-github-002 | Current action default6.1.1, pre-promotion floating baseline6.1.0 and preserved immutablev6.0.0 caveat; release and runtime validator suites. |

Fresh final-source verification is required: pinned Rust1.89.0 full unit+integration, fmt, Clippy-Dwarnings, types, optimized build, strict specs/coverage, existing52release and29CI validator tests, current security audit, independent GPT review, native scoped verification, pre-push, Trust and Attest. Old green tests do not qualify changed source. Hosted CI, human implementation acceptance and dual-OS immutable RC qualification remain later evidence.

## Focused evidence and native watch boundary

The hash-verified old optimized subject a3c2da34 reproduces all six root-file creation failures. The corrected command process matrix, scoped repeated JSON/text batch, flat/unknown classification and two-exact-file ambiguity controls pass in receipt051019-CODEX-de326c. Source selection has a clean eight-fail/eight-pass causal matrix, with ordering and root-alias red-to-green corrections; final61generator and138command tests pass. Watch34focused mode/confinement/fingerprint/rearm controls pass at pre-lint b9f1ce69; independent review proves final f06712ca differs only by an equivalent Clippy let-chain. Fresh full final-source verification remains required.

Native error-to-recovery controls are retained as failures: current052210/052326 and frozen-before052455 both observe a real invalid-spec error then miss restoration. Source phases are not reached in those stronger controls. A comment-only coalesced directory PASS does not establish source-edit event causality. The complete Linux backend process regression now requires a discriminating spec-error/restored-PASS control, source edits, atomic save, later edits, unlink/recreate and a sibling-negative control; hosted execution remains pending. No manual Mac sequence pass is claimed.

## Final frozen local qualification

Receipt052657-CODEX-0afcb8 passes the complete pinned Rust1.89 Fledge verify lane plus external latest-released6.1.0 and optimized-candidate6.1.1 forced strict100% coverage and the current security audit. Final unit2574/2574 and integration462/462 pass with0failures/ignored tests. Source/test/tool194-file hashes match before/after each step; final optimized binary SHA256 2da7ea247cba98756c1fbf14cd72ae57915555ccfebf494dfab067f0ebde140a. The initial Clippy collapsible-if failure is preserved; its exact short-circuit-equivalent correction was independently reviewed before this successful rerun. Linux-only actual-backend sequence still requires hosted execution; failed Mac control comparisons remain explicit.
