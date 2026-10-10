---
change: split-configuration-loading-parsing-and-discovery-into-focused-private-modules
artifact: testing
---

# Testing

Preserve the original 98 configuration tests and compare their compiled inventory. Independently verify function bodies, type declarations, constants, literals, attributes and cfg gates against main 815690d. Run the focused suite, full unit/integration suites with exact known-baseline comparisons, production and all-target clippy, cargo fmt/check/release, strict full coverage and pre-push. Use the real SDD check and check --commit lifecycle. Record unavailable Fledge/Attest/Ruby and Windows execution explicitly rather than claim them passed.

## Observed evidence

- Before and after extraction: 98 focused configuration tests passed; compiled test names are identical.
- Independent AST and literal review: all 155 function/method bodies, 98 tests, 7 constants, 2 traits, 4 impls, original attributes and cfg guards, and the 19-symbol public/crate-visible facade are preserved. The only declaration-format delta is an optional trailing comma introduced by rustfmt.
- Full unit suite: 2338 passed, 192 failed, with exactly the same 192 failure names as the pristine main baseline.
- Full integration suite: 425 passed, 13 failed, with exactly the same 13 failure names as the pristine main baseline. Existing failures are constrained-environment Git/ancestry behavior; no marker removal or workaround was used.
- Production cargo check, production clippy with warnings denied, and formatting pass. All-target clippy reports the same 31 diagnostics as pristine main, with none added or removed; unrelated files were not edited.
- Independent engineering review found no source correctness, API, platform import, or security-boundary regressions. It does not constitute human implementation approval.

- Optimized release check and build pass. Strict coverage passes all 62 specs with zero warnings and 100% file/LOC coverage. The mandatory pre-push gate passes.
- All 52 release-candidate contract tests pass. The required-CI-gate script reaches 5 successful tests but reports 9 errors because Ruby/Psych is unavailable; it is not a green gate.
- Fledge trust verification, Attest provenance, and Ruby are unavailable (executables not found). No new runtimes or dependencies were installed and no trust gate was bypassed. Only the installed x86_64-unknown-linux-gnu target is exercised; Windows is not installed.

Initial real scoped lifecycle verification passed and materialized config spec version 28. The local checkpoint will be recorded with the real `change check --commit` flow. Human implementation review, publication, CI, acceptance and archival are not claimed.
