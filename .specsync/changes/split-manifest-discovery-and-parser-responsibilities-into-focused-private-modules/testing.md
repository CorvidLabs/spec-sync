---
change: split-manifest-discovery-and-parser-responsibilities-into-focused-private-modules
artifact: testing
---

# Testing

Preserve all original function bodies and literal bytes with a mechanical raw-token comparison; when rustfmt changes optional punctuation, compare the same body in a common formatting context and explicitly review that exception. Retain all existing tests, helper behavior and platform cfgs. Baseline cargo test manifest:: passes 59 Linux tests; native Windows runtime remains unavailable. Run formatter, debug/release cargo check, focused manifest tests, full unit and integration suites, clippy with warnings denied, release build, strict 100 percent coverage, mandatory pre-push and real change check --commit. Compare full-suite failure names against pristine main, never weaken sandbox or Git-discovery checks. Ruby/Psych and fledge/Attest remain unavailable; do not claim full trust. Independent technical review is separate from genuine human implementation review.

## Focused Refactor Evidence

The facade is reduced from 5465 to 149 lines. All 22 Rust files are below 1000 lines; the largest production owner has 762 lines and the largest test file 626. All 60 declared tests are preserved (59 on Linux and one Windows-only), with their exact test/cfg attributes. All 70 test/helper names and 104 raw literals are retained. The 59 Linux tests pass, including the two actual one-test child runs under a 64-descriptor limit. Debug cargo check, cargo fmt --check and git diff --check pass.

Mechanical comparison preserves all 180 original function/method/test bodies in a common rustfmt context. Raw comparison explicitly identifies three optional call-comma changes in moved tests/helpers; no production body or literal byte changed. Only two stable-path facade wrappers were added. The ManifestAccess trait, including its three bodyless declarations, retains its complete token sequence apart from private visibility.

Independent final technical review found no blocking issues. It checked all original declarations and attributes, public/crate API paths, fixed parser/merge order, Gradle guard and includeBuild semantics, retained identities, workspace charging/caching, listing verification/release, all 60 test declarations and exact canonical file coverage. This is technical review, not a human implementation acceptance.

## Full-Suite Baseline Comparison

The refactored full unit suite has 2338 passes and 192 failures. The complete integration suite has 425 passes and 13 failures. Both failure-name sets exactly match the pristine main 815690d baseline. No detection or security guard was weakened. Full-suite success is not claimed: this environment retains the baseline lifecycle/Git-discovery failures, Ruby/Psych is unavailable for its CI checks, and fledge/Attest trust tooling is unavailable. Windows runtime was not executed.

## Final Available Gates

Debug and release cargo check, cargo fmt --check, cargo clippy --locked -- -D warnings and cargo build --release --locked passed. The mandatory pre-push gate passed; strict validation has 62 passing specs, zero warnings and 118/118 source files covered. Manifest spec quality is 100/A. Source and documents are prepared for the normal scoped lifecycle materialize/verify commit, with all baseline/full-trust limitations retained above. No human implementation review, publication or merge is claimed.
