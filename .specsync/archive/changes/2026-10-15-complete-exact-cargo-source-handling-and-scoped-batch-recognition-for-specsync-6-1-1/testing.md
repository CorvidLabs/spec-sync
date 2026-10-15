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

## Completed hosted product qualification at5f0c9c69

The actual completed Linux test job116269062060 in run38688508584 passed2,577unit and464integration tests with0failures/ignored tests. Its log explicitly records `commands::watch_exact_source_edits_survive_atomic_save_and_recreation ... ok`, covering real spec-error/recovery, source edits, atomic replacement, recreation and sibling-negative behavior. Exact-head coverage, audit, Trust, scoped review and implementation-ready gates passed. Prior local Mac native control failures remain disclosed; no complete manual Mac event sequence is claimed.

The four original review threads are resolved on that evidence. A later review found a separate partial-denominator bug after Cargo discovery loss, tracked in draft `refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1`. The5f0qualification above is not reused as final qualification of its forthcoming source correction. The two cmd_generate/github3md context repairs are separately reviewed. Human implementation acceptance and finalization remain pending.

## Completed final frozen local qualification

Receipt061410-CODEX-e7709d exited0 after857.629seconds. Pinned Rust1.89.0 complete Fledge verify passed2,582unit and462Mac integration tests with0failures/ignored, formatting, Clippy with warnings denied, types, optimized build,62strict specs and forced100%file/LOC coverage,52release-validator and29CI-gate tests. Latest released6.1.0 and fresh candidate6.1.1 strict checks both passed. The current security audit reports0vulnerabilities and no new ignore entries; official advisory HEADedb58f5974b83eb8302332821cb6d389777aa351 matches the cached1329-advisory snapshot.

All194runtime/test/tool inputs match the freeze before and after each step. Final optimized binary SHA256cfad05c5ce1fd5d03b4d4bfb2689c086787ca5c7de0a8f087d312ab13ce546de also passes the same16real text/JSON coverage/check CLI cases in receipt062835-CODEX-def8d2. Independent source review6c518139 reports0source findings; source65a9616c and tests83bd700e match the reviewed freeze. Existing test-only unused_mut warning in unrelated substantive_plan.rs is preserved; it was not introduced or suppressed by this repair.

No new human approval or implementation acceptance is claimed. Native scoped verification, new product publication/exact-head CI, sequential archive, normal merge and immutable Ubuntu/macOS release qualification remain later gates. Prior Mac native watcher-control failures in old and new subjects remain disclosed; no complete manual Mac source event sequence is claimed.
