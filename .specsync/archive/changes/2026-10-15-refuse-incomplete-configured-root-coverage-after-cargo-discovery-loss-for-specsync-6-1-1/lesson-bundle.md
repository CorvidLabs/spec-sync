# Lesson bundle — refuse-incomplete-configured-root-coverage-after-cargo-discovery-loss-for-specsync-6-1-1

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse incomplete configured root coverage after Cargo discovery loss for SpecSync 6.1.1
- **Kind**: BugFix
- **Specs**: validator
- **Paths**: src/validator/consumed_by_re.rs, src/validator/tests.rs, specs/validator/
- **Acceptance**: Coverage and check must never report a complete partial inventory when an eligible selected regular root source file loses checked Cargo exact-target authority, including a previously inferred selection whose manifest disappears between discovery phases. Valid checked Cargo files retain existing coverage; malformed, missing or changed manifests with an unrecognized eligible selected regular root file fail inconclusive instead of omitting it. Explicit directory-only fallback, extension/test/exclusion filtering, no-follow confinement, exact-file boundaries and existing input budgets remain intact. Preserve the reproduced nonempty50-to-false100 before case, prove corrected text/JSON controls and pass independent review and final pinned local verification before genuine acceptance and release gates.

## Evidence

- Verification commit: `7a6f412a26e9e6a59a252a3a6b196f0540a5fdf9`
- Base commit: `5f0c9c69a0b40963c7a62c06feb74854e4aa8341`
- Verified by: `specsync check --spec validator`

## From the change's context.md

# Context

The source5f0c9c69 already fixes scoped batch recognition, exact-file creation and watcher admission. Its original four review threads are resolved, with actual hosted2577unit/464integration tests and native Linux event evidence. A subsequent automated review revealed a separate retained coverage defect before merge. The original archived package and the active four-fix definition approval remain historical facts; neither is relabeled as acceptance of this repair.

Same explicit source_dirs=[lib.rs,src], with src covered and root lib.rs uncovered: valid Cargo reports1/2files50% and exit1; malformed or absent Cargo reports1/1files100% and exit0. Actual causal receipt055930-CODEX-476eef and immutable before optimized SHA2da7ea247cba98756c1fbf14cd72ae57915555ccfebf494dfab067f0ebde140a are preserved externally. A successful harness exit means reproduction of erroneous behavior, not a fixed product.

This package remains a draft until genuine human definition acceptance. No future implementation inspection, CI pass, review or release is claimed.

## From the change's design.md

# Design

Retain the existing checked-Cargo-only admission of exact regular root source files. A configured eligible regular file cannot silently become a skipped directory when manifest discovery loses its exact-target authority: return an ordinary inconclusive coverage error instead. This also covers absent manifests, which can legitimately produce an empty discovery result without a parse error. Do not turn an explicit file into a recursive project-root scan or admit arbitrary configured files.

Keep directory-only configured fallback, filtering and existing capability/identity checks. Excluded or non-source candidates retain existing filtering order. Do not open a symlink, outside root, nonregular object or unknown source merely to recover a denominator. Valid checked Cargo targets and ordinary recursive directories retain their stated selections and current budgets.

The user-visible failure uses existing text/JSON error conventions. No new CLI option, dependency, persisted format, owner mapping or lifecycle bypass is introduced. Independent review compares the final source/test hashes and actual before/after receipts.

## From the change's testing.md

# Testing

The before fixture proves valid Cargo yields50%1/2 and exit1 while malformed/missing Cargo yields false100%1/1 and exit0; both coverage and check are exercised. Existing explicit[src]-only directory fallback remains100%1/1, and malformed Cargo plus a symlink selection already fails inconclusive without outside bytes.

Require the actual retained-config inference then manifest-removal regression, plus failure-before/pass-after targeted tests, eligible-file missing/malformed/changed-valid authority, valid Cargo counting, directory-only fallback, filters and symlink/escape/special-file safety. Text and JSON commands must report an inconclusive failure rather than a smaller successful denominator. Avoid vacuous zero-inventory proofs.

## Requirement evidence

| Requirement | Required evidence |
|-------------|-------------------|
| REQ-validator-044 | Actual mixed covered-directory/uncovered-root-file before cases; corrected missing/malformed Cargo inconclusive text/JSON gates; unchanged valid Cargo exact-target counting; directory-only and filtering/confinement controls; independent final review and frozen complete qualification. |

Full pinned formatting, Clippy, types, unit/integration, optimized build, strict contracts/coverage and existing release/CI validators must qualify final corrected source. Current security audit remains required. Prior5f0 product CI, Linux watcher runtime proof and full local results are historical evidence, not qualification of this additional repair. Hosted exact-head checks and immutable Ubuntu/macOS release qualification remain later gates.

## Completed final frozen local qualification

Receipt061410-CODEX-e7709d exited0 after857.629seconds. Pinned Rust1.89.0 complete Fledge verify passed2,582unit and462Mac integration tests with0failures/ignored, formatting, Clippy with warnings denied, types, optimized build,62strict specs and forced100%file/LOC coverage,52release-validator and29CI-gate tests. Latest released6.1.0 and fresh candidate6.1.1 strict checks both passed. The current security audit reports0vulnerabilities and no new ignore entries; official advisory HEADedb58f5974b83eb8302332821cb6d389777aa351 matches the cached1329-advisory snapshot.

All194runtime/test/tool inputs match the freeze before and after each step. Final optimized binary SHA256cfad05c5ce1fd5d03b4d4bfb2689c086787ca5c7de0a8f087d312ab13ce546de also passes the same16real text/JSON coverage/check CLI cases in receipt062835-CODEX-def8d2. Independent source review6c518139 reports0source findings; source65a9616c and tests83bd700e match the reviewed freeze. Existing test-only unused_mut warning in unrelated substantive_plan.rs is preserved; it was not introduced or suppressed by this repair.

No new human approval or implementation acceptance is claimed. Native scoped verification, new product publication/exact-head CI, sequential archive, normal merge and immutable Ubuntu/macOS release qualification remain later gates. Prior Mac native watcher-control failures in old and new subjects remain disclosed; no complete manual Mac source event sequence is claimed.

## Where these lessons go

- `specs/validator/validator.3md`
