---
change: CHG-0137-coverage-must-not-invent-a-module-over-files-that-are-all-mapped
artifact: tasks
---

# Tasks

- [x] Reproduce from scratch in a throwaway fixture: five language-specific specs each mapping
      one source file, and confirm coverage invents a parent that does not exist.
- [x] Confirm it reproduces on this repository, and identify where each phantom name came from.
- [x] Enumerate every site that derives a module name from a path BEFORE changing any of them,
      and give each a disposition. Four assemble `unspecced_modules`; the rest map a name to an
      already-chosen module and are unaffected.
- [x] Fix all four with one shared rule rather than four local patches.
- [x] Wire `ManifestModule::source_paths`, previously dead code, so the manifest derivation has
      files to judge against; remove its `#[allow(dead_code)]`.
- [x] Keep `owned == 0` reporting: a module owning nothing measurable stays visible.
- [x] Prove the coverage percentages are unchanged, so the fix removes a false claim rather than
      widening what counts as covered.
- [x] Build the unfixed binary from a separate checkout of `origin/main`, verify `git diff
      --numstat -- src/` is 0 lines and the build exited 0, then run the new tests against it.
- [x] Confirm the new tests FAIL on unfixed and PASS on fixed, and that the vacuity controls pass
      on both.
- [x] Verify sandbox gate 061 on both binaries rather than inferring it from the fix working.
- [x] File the adjacent defects separately: #610, #611, #612, #613.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] CHANGELOG entry.
- [x] Semantic deltas for both affected modules; do not hand-edit `specs/`.
