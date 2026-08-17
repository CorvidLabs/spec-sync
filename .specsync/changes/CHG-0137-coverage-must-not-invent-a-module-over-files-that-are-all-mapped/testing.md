---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-validator-041 | Two tests fail on an unfixed binary built from a separate checkout of `origin/main` (`a_fully_mapped_stem_is_not_reported_as_a_module_without_a_spec`, `a_fully_mapped_directory_is_not_reported_as_a_module_without_a_spec`) and pass on the fixed one. Three further tests pass on **both** and are the vacuity controls: an unmapped sibling keeps the stem reported, an unmapped file keeps the directory reported, and a directory holding nothing measurable stays reported. Sandbox gate 061 goes rc=1 to rc=0 with its own not-a-mute control green. Coverage figures are asserted unchanged, so the candidate is suppressed without altering the measurement. The all-four-derivations criterion is covered by the manifest and configured-module controls, since the phantom on this repository came from the manifest site rather than the reported one |
| REQ-manifest-018 | `ManifestModule::source_paths` is populated by Cargo, Swift and Gradle discovery including the single-project fallback, and its `#[allow(dead_code)]` is removed — the field is now load-bearing, so a discovery path that failed to populate it would fail the validator tests above rather than compile silently. `src/manifest.rs` marker counts are unchanged at 56 `#[test]` and 174 `fn`, so no test was displaced |

## Suite

    cargo test                    rc=0    2283 unit passed, 379 integration passed, 0 failed
    cargo clippy -- -D warnings   rc=0
    cargo fmt --check             rc=0

`cargo clippy --all-targets` is rc=101, identical to unmodified `main` — the finding lists were
diffed with line numbers stripped and are byte-identical. Pre-existing debt, filed as #608.

Marker counts, taken before and after rather than read off a diff: `src/validator.rs` 55 to 62
`#[test]` and 153 to 172 `fn` — 7 tests, 2 test helpers, 10 production functions, exactly what was
added. `src/manifest.rs` unchanged at 56 and 174.

## Discrimination

The unfixed binary was built from a **separate clone** of `origin/main`, never by reverting files
in the working tree. That method was adopted after a reverted-file build failed to compile earlier
in this release, leaving drills running against the still-fixed binary and passing — a false
proof caught only by checking the build's exit code. Here both were checked explicitly:
`unfixed_build_rc=0`, `unfixed_testbuild_rc=0`, and `git diff --numstat -- src/` in that clone
returned 0 lines.

    UNFIXED  rc=101   2 failed, 3 passed
      coverage invented module `strutil` over files that are all mapped: ["strutil"]
      coverage invented module `textkit` over files that are all mapped: ["textkit"]

    FIXED    rc=0     5 passed

The 3 that pass on unfixed are the vacuity controls, which is the point: they must not
discriminate.

## Gate 061, run explicitly

    UNFIXED  rc=1   pass=3  fail/pending=1
    FIXED    rc=0   pass=4  fail=0

Run against both binaries rather than inferred from the fix working. Earlier in this release,
gates 046 and 047 stayed red after their fixes merged and needed a further change on top, so
"the fix works" and "the gate flips" are treated as separate assertions.

## On this repository

    before   modules: ['specsync', 'change_tests']    files_covered: 106 / 106
    after    modules: []                              files_covered: 106 / 106

The unchanged denominator is the load-bearing half. A fix that had over-suppressed would have
moved the coverage figure too.
